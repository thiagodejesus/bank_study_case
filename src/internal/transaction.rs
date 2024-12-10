use serde::Deserialize;

use super::{account::{account::AccountManager, domain::Account}, error::BankError};

#[derive(Debug)]
pub struct TransactionError {
    message: String,
    status: axum::http::StatusCode,
}

impl BankError for TransactionError {
    fn message(&self) -> &str {
        &self.message
    }
    fn status(&self) -> &axum::http::StatusCode {
        &self.status
    }
}

pub struct TransactionManager {
    db_pool: sqlx::PgPool,
}

impl TransactionManager {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create_transaction(&self, transaction: Transaction) -> Result<(), impl BankError> {
        match transaction {
            Transaction::Deposit {
                amount,
                destination,
            } => {
                println!(
                    "Deposit: amount={:?}, destination={:?}",
                    amount, destination
                );
                let result = sqlx::query!(
                    "INSERT INTO transaction (account_id, amount, type) VALUES ($1, $2, $3)",
                    destination.id,
                    amount,
                    "deposit"
                )
                .execute(&self.db_pool)
                .await;

                match result {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(TransactionError {
                            message: e.to_string(),
                            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        });
                    }
                }
            }
            Transaction::Withdraw { amount, origin } => {
                println!("Withdraw: amount={:?}, origin={:?}", amount, origin);

                // On a transaction, get the balance, if its enough, proceed with the transaction
                // if not, return an error

                let mut tx = self.db_pool.begin().await.unwrap();

                let balance = AccountManager::get_balance(&origin, &mut tx).await.unwrap();

                if balance < amount.into() {
                    tx.rollback().await.unwrap();
                    return Err(TransactionError {
                        message: "Insufficient funds".to_string(),
                        status: axum::http::StatusCode::BAD_REQUEST,
                    });
                }

                let result = sqlx::query!(
                    "INSERT INTO transaction (account_id, amount, type) VALUES ($1, $2, $3)",
                    origin.id,
                    amount,
                    "withdraw"
                )
                .execute(&self.db_pool)
                .await;

                match result {
                    Ok(_) => {
                        tx.commit().await.unwrap();
                        return Ok(());
                    }
                    Err(_) => {
                        tx.rollback().await.unwrap();
                        return Err(TransactionError {
                            message: "Error on transaction".to_string(),
                            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        });
                    }
                }
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
            }
        };

        return Err(TransactionError {
            message: "Invalid transaction".to_string(),
            status: axum::http::StatusCode::BAD_REQUEST,
        });
    }
}

#[derive(Deserialize, Debug)]
pub enum Transaction {
    Deposit {
        amount: i64,
        destination: Account,
    },
    Withdraw {
        amount: i64,
        origin: Account,
    },
    Transfer {
        amount: i64,
        origin: Account,
        destination: Account,
    },
}
