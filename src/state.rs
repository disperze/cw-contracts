use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use crate::msg::LockResponse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub max_lock_time: u64,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const LOCKS: Map<&Addr, LockResponse> = Map::new("locks");
