use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Coin};
use cw_storage_plus::{Item, Map};
use cw20::Cw20CoinVerified;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub max_lock_time: u64,
    pub current: u64,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Lock {
    pub owner: Addr,
    pub create: Timestamp,
    pub expire: Timestamp,
    pub funds: GenericBalance,
    pub complete: bool,
}

pub const STATE: Item<State> = Item::new("state");
pub const LOCKS: Map<&str, Lock> = Map::new("locks");
