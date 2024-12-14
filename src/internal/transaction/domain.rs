use serde::Deserialize;

use crate::internal::account::domain::Account;

#[derive(Deserialize, Debug)]
pub enum Transaction {
    Deposit {
        amount: u32,
        destination: Account,
    },
    Withdraw {
        amount: u32,
        origin: Account,
    },
    Transfer {
        amount: u32,
        origin: Account,
        destination: Account,
    },
}
