use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_address: String,
    pub initial_value: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetBalance { address: String, value: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    Account { address: String },
}
