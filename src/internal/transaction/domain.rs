use serde::Deserialize;

use crate::internal::account::domain::Account;

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
