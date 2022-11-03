use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub native_coin: String,
}

pub const STATE: Item<State> = Item::new("state");
