use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::balance::GenericBalance;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub max_lock_time: u64,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Lock {
    pub create: Timestamp,
    pub expire: Timestamp,
    pub funds: GenericBalance,
}

pub const STATE: Item<State> = Item::new("state");
pub const LOCKS: Map<(&Addr, String), Lock> = Map::new("locks");
