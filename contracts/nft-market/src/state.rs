use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use cosmwasm_std::{Addr, Api, Coin, StdResult, Storage};
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

pub fn increment_offerings(store: &mut dyn Storage) -> Result<u64, ContractError> {
    let mut num = 0;
    STATE.update(store, |mut state| -> Result<_, ContractError> {
        state.num_offerings += 1;
        num = state.num_offerings;
        Ok(state)
    })?;

    Ok(num)
}

pub fn get_fund(funds: Vec<Coin>, denom: String) -> Result<Coin, ContractError> {
    for fund in funds.into_iter() {
        if fund.denom == denom {
            return Ok(fund);
        }
    }

    Err(ContractError::InsufficientFunds {})
}

pub fn maybe_addr(api: &dyn Api, human: Option<String>) -> StdResult<Option<Addr>> {
    human.map(|x| api.addr_validate(&x)).transpose()
}
