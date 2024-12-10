use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub(crate) id: Uuid,
    pub(crate) number: i64,
}

impl Account {
    pub fn new(number: i64) -> Self {
        Self {
            id: Uuid::now_v7(),
            number,
        }
    }

    pub fn from_existing(id: Uuid, number: i64) -> Self {
        Self { id, number }
    }

    pub fn number(&self) -> &i64 {
        &self.number
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}
