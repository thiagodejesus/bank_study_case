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

    pub async fn get_account_from_number(
        &self,
        number: i64,
    ) -> Result<Account, Box<dyn BankError>> {
        let account = sqlx::query_as!(
            Account,
            "SELECT id, number FROM account WHERE number = $1",
            number
        )
        .fetch_optional(self.db_pool)
        .await;

        match account {
            Ok(account) => match account {
                Some(account) => Ok(account),
                None => Err(Box::new(AccountError::new(
                    format!("Account [{}] not found", number),
                    axum::http::StatusCode::NOT_FOUND,
                ))),
            },
            Err(e) => Err(Box::new(AccountError::new(
                e.to_string(),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))),
        }
    }

    pub async fn create_account(&self) -> Result<Account, Box<dyn BankError>> {
        let mut tx = match self.db_pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                println!("Error starting database transaction: {}", e);
                return Err(Box::new(AccountError::new(
                    "An unexpected error happened, please try again".to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        };

        let conn = match tx.acquire().await {
            Ok(conn) => conn,
            Err(e) => {
                println!("Error getting database connection: {}", e);
                return Err(Box::new(AccountError::new(
                    "An unexpected error happened, please try again".to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        };

        let latest_number = sqlx::query!("SELECT number FROM account ORDER BY number DESC LIMIT 1")
            .fetch_optional(&mut *conn)
            .await;

        let latest_number = match latest_number {
            Ok(latest_number) => latest_number,
            Err(e) => {
                println!("Error getting latest account number: {}", e);
                return Err(Box::new(AccountError::new(
                    "An unexpected error happened, please try again".to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        };

        let account: Account = Account::new(match latest_number {
            None => 1,
            Some(latest_number) => latest_number.number + 1,
        });

        let res = sqlx::query!(
            "INSERT INTO account (id, number) VALUES ($1, $2)",
            account.id(),
            account.number()
        )
        .execute(conn)
        .await;

        match res {
            Ok(_) => {
                match tx.commit().await {
                    Ok(_) => {
                        return Ok(account);
                    }
                    Err(e) => {
                        println!("Error committing transaction: {}", e);
                        return Err(Box::new(AccountError::new(
                            "An unexpected error happened, please try again".to_string(),
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )));
                    }
                };
            }
            Err(e) => {
                if let Err(e) = tx.rollback().await {
                    println!("Error rolling back transaction: {}", e);
                }

                return Err(Box::new(AccountError::new(
                    e.to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        }
    }

    pub async fn list_accounts(&self) -> Result<Vec<Account>, Box<dyn BankError>> {
        let accounts = sqlx::query_as!(Account, "SELECT id, number FROM account")
            .fetch_all(self.db_pool)
            .await;

        match accounts {
            Ok(accounts) => Ok(accounts),
            Err(e) => Err(Box::new(AccountError::new(
                e.to_string(),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))),
        }
    }

    pub async fn get_balance(
        account: &Account,
        conn: &mut sqlx::PgConnection,
    ) -> Result<BigDecimal, Box<dyn BankError>> {
        let balance = sqlx::query!(
            "SELECT SUM(amount) FROM transaction WHERE account_id = $1",
            account.id()
        )
        .fetch_one(conn)
        .await;

        match balance {
            Ok(balance) => {
                let balance = balance.sum.unwrap_or(0.into());
                Ok(balance)
            }
            Err(_) => Err(Box::new(AccountError::new(
                "Failed to get balance".to_string(),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::test_util::get_conn_with_new_db;

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

        assert_eq!(balance, 0.into());
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
