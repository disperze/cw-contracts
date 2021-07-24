use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::LockResponse;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub max_lock_time: u64,
    pub current: u64,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const LOCKS: Map<&str, LockResponse> = Map::new("locks");
