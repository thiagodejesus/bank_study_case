use crate::internal::{
    account::{account::AccountManager, domain::Account},
    error::BankError,
    transaction::error::TransactionError,
};

use super::domain::Transaction;

pub struct TransactionManager<'a> {
    db_pool: &'a sqlx::PgPool,
}

impl<'a> TransactionManager<'a> {
    pub fn new(db_pool: &'a sqlx::PgPool) -> Self {
        Self { db_pool }
    }

    async fn create_deposit(
        amount: u32,
        destination: &Account,
        conn: &mut sqlx::PgConnection,
    ) -> Result<(), Box<dyn BankError>> {
        let amount_parsed: i64 = amount.into();
        let result = sqlx::query!(
            "INSERT INTO transaction (account_id, amount, type) VALUES ($1, $2, $3)",
            destination.id,
            amount_parsed,
            "deposit"
        )
        .execute(conn)
        .await;

        match result {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(Box::new(TransactionError::new(
                    e.to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        }
    }

    async fn create_withdraw(
        amount: u32,
        origin: &Account,
        conn: &mut sqlx::PgConnection,
    ) -> Result<(), Box<dyn BankError>> {
        let amount_parsed: i64 = amount.into();
        let result = sqlx::query!(
            "INSERT INTO transaction (account_id, amount, type) VALUES ($1, $2, $3)",
            origin.id,
            -amount_parsed,
            "withdraw"
        )
        .execute(conn)
        .await;

        match result {
            Ok(_) => {
                return Ok(());
            }
            Err(_) => {
                return Err(Box::new(TransactionError::new(
                    "Error on transaction".to_string(),
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        }
    }

    pub async fn create_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<(), Box<dyn BankError>> {
        match transaction {
            Transaction::Deposit {
                amount,
                destination,
            } => {
                println!(
                    "Deposit: amount={:?}, destination={:?}",
                    amount, destination
                );

                let mut conn = match self.db_pool.acquire().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        println!("Error getting connection: {}", e);
                        return Err(Box::new(TransactionError::new(
                            "An unexpected error happened, please try again".to_string(),
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )));
                    }
                };
                return TransactionManager::create_deposit(amount, &destination, &mut conn).await;
            }
            Transaction::Withdraw { amount, origin } => {
                println!("Withdraw: amount={:?}, origin={:?}", amount, origin);

                // On a transaction, get the balance, if its enough, proceed with the transaction
                // if not, return an error

                let mut tx = match self.db_pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("Error starting database transaction: {}", e);
                        return Err(Box::new(TransactionError::new(
                            "An unexpected error happened, please try again".to_string(),
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )));
                    }
                };

                let balance = AccountManager::get_balance(&origin, &mut tx).await?;

                if balance < amount.into() {
                    if let Err(e) = tx.rollback().await {
                        println!("Error rolling back transaction: {}", e);
                    }
                    return Err(Box::new(TransactionError::new(
                        "Insufficient funds".to_string(),
                        axum::http::StatusCode::BAD_REQUEST,
                    )));
                }

                match TransactionManager::create_withdraw(amount, &origin, &mut tx).await {
                    Ok(_) => {
                        if let Err(err) = tx.commit().await {
                            println!("Error committing transaction: {}", err);
                            return Err(Box::new(TransactionError::new(
                                "An unexpected error happened, please try again".to_string(),
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            )));
                        }

                        return Ok(());
                    }
                    Err(e) => {
                        if let Err(e) = tx.rollback().await {
                            println!("Error rolling back transaction: {}", e);
                        }
                        return Err(e);
                    }
                };
            }
            Transaction::Transfer {
                amount,
                origin,
                destination,
            } => {
                println!(
                    "Transfer: amount={:?}, origin={:?}, destination={:?}",
                    amount, origin, destination
                );

                let mut tx = match self.db_pool.begin().await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("Error starting database transaction: {}", e);
                        return Err(Box::new(TransactionError::new(
                            "An unexpected error happened, please try again".to_string(),
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )));
                    }
                };

                let balance = AccountManager::get_balance(&origin, &mut tx).await?;

                if balance < amount.into() {
                    if let Err(e) = tx.rollback().await {
                        println!("Error rolling back transaction: {}", e);
                    }
                    return Err(Box::new(TransactionError::new(
                        "Insufficient funds".to_string(),
                        axum::http::StatusCode::BAD_REQUEST,
                    )));
                }

                let withdraw_result =
                    TransactionManager::create_withdraw(amount, &origin, &mut tx).await;
                let deposit_result =
                    TransactionManager::create_deposit(amount, &destination, &mut tx).await;

                match (withdraw_result, deposit_result) {
                    (Ok(_), Ok(_)) => {
                        match tx.commit().await {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error committing transaction: {}", e);
                                return Err(Box::new(TransactionError::new(
                                    "An unexpected error happened, please try again".to_string(),
                                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                )));
                            }
                        }
                        return Ok(());
                    }
                    _ => {
                        if let Err(e) = tx.rollback().await {
                            println!("Error rolling back transaction: {}", e);
                        }
                        return Err(Box::new(TransactionError::new(
                            "Error on transaction".to_string(),
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )));
                    }
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::test_util::get_conn_with_new_db;
    use crate::internal::transaction::domain::Transaction;

    #[tokio::test]
    async fn test_create_transaction_deposit() {
        let database = get_conn_with_new_db().await;

        let transaction_manager = TransactionManager::new(database.get_pool());

        let account_manager = AccountManager::new(&database.get_pool());

        let account = account_manager.create_account().await.unwrap();

        let transaction = Transaction::Deposit {
            amount: 100,
            destination: account.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;
        assert!(result.is_ok());

        let mut conn = database.get_pool().acquire().await.unwrap();
        let balance = AccountManager::get_balance(&account, &mut conn)
            .await
            .unwrap();

        assert_eq!(balance, 100.into());
    }

    #[tokio::test]
    async fn test_create_transaction_withdraw() {
        let database = get_conn_with_new_db().await;
        let db_pool = database.get_pool();
        let transaction_manager = TransactionManager::new(db_pool);

        let account_manager = AccountManager::new(db_pool);

        let account = account_manager.create_account().await.unwrap();

        let transaction = Transaction::Deposit {
            amount: 100,
            destination: account.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_ok());

        let transaction = Transaction::Withdraw {
            amount: 50,
            origin: account.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_ok());

        let balance = AccountManager::get_balance(&account, &mut db_pool.acquire().await.unwrap())
            .await
            .unwrap();

        assert_eq!(balance, 50.into());

        let transaction = Transaction::Withdraw {
            amount: 25,
            origin: account.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_ok());

        let balance = AccountManager::get_balance(&account, &mut db_pool.acquire().await.unwrap())
            .await
            .unwrap();

        assert_eq!(balance, 25.into());
    }

    #[tokio::test]
    async fn test_create_transaction_withdraw_insufficient_funds() {
        let database = get_conn_with_new_db().await;
        let db_pool = database.get_pool();

        let transaction_manager = TransactionManager::new(db_pool);

        let account_manager = AccountManager::new(db_pool);

        let account = account_manager.create_account().await.unwrap();

        let transaction = Transaction::Deposit {
            amount: 100,
            destination: account.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_ok());

        let transaction = Transaction::Withdraw {
            amount: 150,
            origin: account,
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_transaction_transfer() {
        let database = get_conn_with_new_db().await;
        let db_pool = database.get_pool();

        let transaction_manager = TransactionManager::new(db_pool);

        let account_manager = AccountManager::new(db_pool);

        let account_origin = account_manager.create_account().await.unwrap();
        let account_destination = account_manager.create_account().await.unwrap();

        let transaction = Transaction::Deposit {
            amount: 100,
            destination: account_origin.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_ok());

        let transaction = Transaction::Transfer {
            amount: 25,
            origin: account_origin.clone(),
            destination: account_destination.clone(),
        };

        let result = transaction_manager.create_transaction(transaction).await;

        assert!(result.is_ok());

        let balance_origin =
            AccountManager::get_balance(&account_origin, &mut db_pool.acquire().await.unwrap())
                .await
                .unwrap();

        let balance_destination = AccountManager::get_balance(
            &account_destination,
            &mut db_pool.acquire().await.unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(balance_origin, 75.into());
        assert_eq!(balance_destination, 25.into());
    }
}
