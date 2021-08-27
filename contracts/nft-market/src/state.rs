use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr, Coin, DepsMut, Response, Storage};
use cw_storage_plus::{Item, Map};
use crate::error::ContractError;

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

pub fn increment_offerings(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let mut num = 0;
    STATE.update(store, |mut state| -> Result<_, ContractError> {
        state.num_offerings += 1;
        num = state.num_offerings;
        Ok(state)
    })?;

    Ok(num)
}
