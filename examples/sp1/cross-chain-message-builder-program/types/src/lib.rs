use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageBuilderProgramInput {
    pub from: String,
    pub to: String,
    pub amount: u64,
}
