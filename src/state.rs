use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub contract: String,
    pub native_coin: String,
    pub min_mint: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
