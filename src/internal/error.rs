use std::fmt::Debug;

pub trait BankError: Debug + Send {
    fn message(&self) -> &str;
    fn status(&self) -> &axum::http::StatusCode; // Should change it to an internal status system, so the internal impl won't depend on axum
}
