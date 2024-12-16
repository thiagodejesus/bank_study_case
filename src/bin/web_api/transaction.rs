use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use bank_case::internal::{
    account::account::AccountManager,
    transaction::{domain::Transaction, transaction::TransactionManager},
};
use serde::Deserialize;

use crate::AppState;

#[axum::debug_handler]
pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(transaction): Json<TransactionDto>,
) -> (StatusCode, String) {
    let account_manager = AccountManager::new(&state.pg_pool);
    let transaction_manager = TransactionManager::new(&state.pg_pool);

    let transaction_parsed: Transaction = match transaction.transaction {
        TransactionEnum::Deposit {
            amount,
            destination,
        } => {
            let account = match account_manager
                .get_account_from_number(destination.into())
                .await
            {
                Ok(account) => account,
                Err(e) => return (e.status().clone(), e.message().to_string()),
            };

            Transaction::Deposit {
                amount,
                destination: account,
            }
        }
        TransactionEnum::Withdraw { amount, origin } => {
            let account = match account_manager.get_account_from_number(origin.into()).await {
                Ok(account) => account,
                Err(e) => return (e.status().clone(), e.message().to_string()),
            };

            Transaction::Withdraw {
                amount,
                origin: account,
            }
        }
        TransactionEnum::Transfer {
            amount,
            origin,
            destination,
        } => {
            // Should not transfer for self
            let origin_account = match account_manager.get_account_from_number(origin.into()).await
            {
                Ok(account) => account,
                Err(e) => return (e.status().clone(), e.message().to_string()),
            };
            let destiny_account = match account_manager
                .get_account_from_number(destination.into())
                .await
            {
                Ok(account) => account,
                Err(e) => return (e.status().clone(), e.message().to_string()),
            };

            Transaction::Transfer {
                amount,
                origin: origin_account,
                destination: destiny_account,
            }
        }
    };

    let result = transaction_manager
        .create_transaction(transaction_parsed)
        .await;

    match result {
        Ok(_) => (StatusCode::CREATED, "".to_string()),
        Err(e) => (e.status().clone(), e.message().to_string()),
    }
}

#[derive(Deserialize)]
pub struct TransactionDto {
    transaction: TransactionEnum,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum TransactionEnum {
    Deposit {
        amount: u32,
        destination: u32,
    },
    Withdraw {
        amount: u32,
        origin: u32,
    },
    Transfer {
        amount: u32,
        origin: u32,
        destination: u32,
    },
}
