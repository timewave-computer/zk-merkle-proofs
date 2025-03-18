use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::COUNTER,
};
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    let addr = deps.api.addr_validate(&msg.initial_address)?;
    COUNTER.save(deps.storage, addr, &msg.initial_value)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("initial_value", msg.initial_value)
        .add_attribute("contract_address", env.contract.address)
        .add_attribute("sender", info.sender.to_string()))
}

pub const MAX_INCREASE_AMOUNT: Uint128 = Uint128::new(100u128);
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::SetBalance { address, value } => {
            execute_insert_balance(deps, info, address, value)
        }
    }
}

pub fn execute_insert_balance(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    value: Uint128,
) -> Result<Response<NeutronMsg>, ContractError> {
    let addr = deps.api.addr_validate(&address)?;
    COUNTER.save(deps.storage, addr, &value)?;
    Ok(Response::default()
        .add_attribute("action", "insert_balance")
        .add_attribute("sender", info.sender))
}

/// ----------------------------- QUERIES ------------------------------------

#[cw_serde]
pub struct CurrentValueResponse {
    pub current_value: Uint128,
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Account { address } => query_current_value(deps, address),
    }
}

pub fn query_current_value(deps: Deps, address: String) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&address)?;
    let current_value = &COUNTER.load(deps.storage, addr)?;
    to_json_binary(&CurrentValueResponse {
        current_value: current_value.clone(),
    })
}
