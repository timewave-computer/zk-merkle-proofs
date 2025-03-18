use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

pub const COUNTER: Map<Addr, Uint128> = Map::new("store");
pub const COUNTER: Map<Uint128, Uint128> = Map::new("store");
