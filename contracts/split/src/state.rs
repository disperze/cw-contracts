use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
    pub native_coin: String,
    pub total_split: Uint128,
    pub owner: Addr,
}

#[cw_serde]
pub struct UserParams {
    pub percent: Decimal,
    pub split: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserParams> = Map::new("users");
