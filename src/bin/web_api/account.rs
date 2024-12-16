use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bank_case::internal::account::{account::AccountManager, domain::Account};
use bigdecimal::BigDecimal;

use crate::AppState;

pub async fn create_account_controller(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Account>), (StatusCode, String)> {
    let account_manager = AccountManager::new(&state.pg_pool);

    match account_manager.create_account().await {
        Ok(account) => Ok((StatusCode::CREATED, Json(account))),
        Err(e) => Err((e.status().clone(), e.message().to_string())),
    }
}

pub async fn list_accounts_controller(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Account>>), (StatusCode, String)> {
    let account_manager = AccountManager::new(&state.pg_pool);

    match account_manager.list_accounts().await {
        Ok(accounts) => Ok((StatusCode::OK, Json(accounts))),
        Err(e) => Err((e.status().clone(), e.message().to_string())),
    }
}

#[axum::debug_handler]
pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(account_number): Path<u32>,
) -> Result<(StatusCode, Json<GetBalanceResponse>), (StatusCode, String)> {
    let mut pg_pool = state.pg_pool.clone().acquire().await.unwrap();
    let account_manager = AccountManager::new(&state.pg_pool);

    let account = match account_manager
        .get_account_from_number(account_number.into())
        .await
    {
        Err(e) => return Err((e.status().clone(), e.message().to_string())),
        Ok(account) => account,
    };

    let balance = AccountManager::get_balance(&account, &mut pg_pool).await;

    match balance {
        Ok(balance) => Ok((StatusCode::OK, Json(GetBalanceResponse { balance }))),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "".to_string())),
    }
}

#[derive(serde::Serialize)]
pub struct GetBalanceResponse {
    balance: BigDecimal,
}
