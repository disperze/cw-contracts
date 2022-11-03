use cosmwasm_schema::cw_serde;

use crate::balance::GenericBalance;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
    pub max_lock_time: u64,
    pub owner: Addr,
}

#[cw_serde]
pub struct Lock {
    pub create: Timestamp,
    pub expire: Timestamp,
    pub funds: GenericBalance,
}

pub const STATE: Item<State> = Item::new("state");
pub const LOCKS: Map<(&Addr, String), Lock> = Map::new("locks");
