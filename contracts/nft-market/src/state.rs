use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr, Coin};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub num_offerings: u64,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Offering {
    pub token_id: String,
    pub contract: Addr,
    pub seller: Addr,
    pub list_price: Coin,
}

pub const STATE: Item<State> = Item::new("state");
pub const OFFERINGS: Map<&str, Offering> = Map::new("offerings");
