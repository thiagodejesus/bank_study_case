use crate::internal::error::BankError;

#[derive(Debug)]
pub struct AccountError {
    message: String,
    status: axum::http::StatusCode,
}

impl AccountError {
    pub fn new(message: String, status: axum::http::StatusCode) -> Self {
        Self { message, status }
    }
}

impl BankError for AccountError {
    fn message(&self) -> &str {
        &self.message
    }
    fn status(&self) -> &axum::http::StatusCode {
        &self.status
    }
}
