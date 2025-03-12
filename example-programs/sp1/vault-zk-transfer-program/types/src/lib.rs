use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferProgramInputs {
    pub from: String,
    pub to: String,
    pub amount: u64,
}
