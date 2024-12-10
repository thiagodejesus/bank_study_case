use std::str::FromStr;

use sqlx::{types::BigDecimal, Acquire};

use crate::internal::error::BankError;

use super::{domain::Account, error::AccountError};

pub struct AccountManager<'a> {
    db_pool: &'a sqlx::PgPool,
}

impl<'a> AccountManager<'a> {
    pub fn new(db_pool: &'a sqlx::PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create_account(&self) -> Result<Account, impl BankError> {
        let mut tx = self.db_pool.begin().await.unwrap();

        let conn = tx.acquire().await.unwrap();

        let latest_number = sqlx::query!("SELECT number FROM account ORDER BY number DESC LIMIT 1")
            .fetch_optional(conn)
            .await
            .unwrap();

        let account: Account = Account::new(match latest_number {
            None => 1,
            Some(latest_number) => latest_number.number + 1,
        });

        let conn = tx.acquire().await.unwrap();

        let res = sqlx::query!(
            "INSERT INTO account (id, number) VALUES ($1, $2)",
            account.id(),
            account.number()
        )
        .execute(conn)
        .await;

        match res {
            Ok(_) => {
                tx.commit().await.unwrap();
                return Ok(account);
            }
            Err(e) => {
                tx.rollback().await.unwrap();
                return Err(AccountError::new(
                    e.to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }

    pub async fn list_accounts(&self) -> Result<Vec<Account>, impl BankError> {
        let accounts = sqlx::query_as!(Account, "SELECT id, number FROM account")
            .fetch_all(self.db_pool)
            .await;

        match accounts {
            Ok(accounts) => Ok(accounts),
            Err(e) => Err(AccountError::new(
                e.to_string(),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }

    pub async fn get_balance(
        account: &Account,
        conn: &mut sqlx::PgConnection,
    ) -> Result<BigDecimal, impl BankError> {
        let balance = sqlx::query!(
            "SELECT SUM(amount) FROM transaction WHERE account_id = $1",
            account.id()
        )
        .fetch_one(conn)
        .await;

        match balance {
            Ok(balance) => {
                let balance = balance
                    .sum
                    .unwrap_or(BigDecimal::from_str("0").expect("Should always work"));
                Ok(balance)
            }
            Err(_) => Err(AccountError::new(
                "Failed to get balance".to_string(),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::test_util::get_conn_with_new_db;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_create_account() {
        let database = get_conn_with_new_db().await;

        let account_manager = super::AccountManager::new(database.get_pool());

        let account = account_manager.create_account().await.unwrap();

        assert_eq!(account.number(), &1);

        let account = account_manager.create_account().await.unwrap();

        assert_eq!(account.number(), &2);
    }

    #[tokio::test]
    async fn test_create_account_with_balance_zero() {
        let database = get_conn_with_new_db().await;

        let account_manager = super::AccountManager::new(database.get_pool());

        let account = account_manager.create_account().await.unwrap();

        let mut conn = database.get_pool().acquire().await.unwrap();

        let balance = super::AccountManager::get_balance(&account, &mut conn)
            .await
            .unwrap();

        assert_eq!(balance, sqlx::types::BigDecimal::from_str("0").unwrap());
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let database = get_conn_with_new_db().await;

        let account_manager = super::AccountManager::new(database.get_pool());

        account_manager.create_account().await.unwrap();

        let accounts = account_manager.list_accounts().await.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].number(), &1);

        account_manager.create_account().await.unwrap();

        let accounts = account_manager.list_accounts().await.unwrap();

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].number(), &1);
        assert_eq!(accounts[1].number(), &2);
    }
}
