use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Timestamp};
use cw20::{Balance, Cw20CoinVerified};
use cw_storage_plus::{Item, Map};

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

impl From<Balance> for GenericBalance {
    fn from(balance: Balance) -> GenericBalance {
        match balance {
            Balance::Native(balance) => GenericBalance {
                native: balance.0,
                cw20: vec![],
            },
            Balance::Cw20(token) => GenericBalance {
                native: vec![],
                cw20: vec![token],
            },
        }
    }
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
