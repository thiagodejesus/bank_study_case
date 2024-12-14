use crate::internal::error::BankError;

#[derive(Debug)]
pub struct TransactionError {
    message: String,
    status: axum::http::StatusCode,
}

impl TransactionError {
    pub fn new(message: String, status: axum::http::StatusCode) -> Self {
        Self { message, status }
    }
}

impl BankError for TransactionError {
    fn message(&self) -> &str {
        &self.message
    }
    fn status(&self) -> &axum::http::StatusCode {
        &self.status
    }
}

impl From<Box<dyn BankError>> for TransactionError {
    fn from(error: Box<dyn BankError>) -> Self {
        Self {
            message: error.message().to_string(),
            status: error.status().clone(),
        }
    }
}
