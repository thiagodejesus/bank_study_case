use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use bank_case::internal::{
    account::domain::Account, error::BankError, transaction::{Transaction, TransactionManager}
};
use serde::Deserialize;

use crate::AppState;

#[axum::debug_handler]
pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(transaction): Json<TransactionDto>,
) -> (StatusCode, String) {
    return (StatusCode::NOT_IMPLEMENTED, "".to_string());
    // let transaction_manager = TransactionManager::new(state.pg_pool.clone());

    // let transaction_parsed: Transaction = match transaction.transaction {
    //     TransactionEnum::Deposit {
    //         amount,
    //         destination,
    //     } => {
    //         let account = Account {
    //             number: destination,
    //         };
    //         Transaction::Deposit {
    //             amount,
    //             destination: account,
    //         }
    //     }
    //     TransactionEnum::Withdraw { amount, origin } => {
    //         let account = Account { number: origin };
    //         Transaction::Withdraw {
    //             amount,
    //             origin: account,
    //         }
    //     }
    // };

    // let result = transaction_manager
    //     .create_transaction(transaction_parsed)
    //     .await;

    // match result {
    //     Ok(_) => (StatusCode::CREATED, "".to_string()),
    //     Err(e) => (e.status().clone(), e.message().to_string()),
    // }
}

#[derive(Deserialize)]
pub struct TransactionDto {
    transaction: TransactionEnum,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum TransactionEnum {
    Deposit { amount: i64, destination: i64 },
    Withdraw { amount: i64, origin: i64 },
}
