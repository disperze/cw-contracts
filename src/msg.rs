use cosmwasm_std::{Coin, Timestamp, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Max lock time in seconds
    pub max_lock_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Lock funds until expire timestamp
    Lock { expire: Timestamp },
    /// Unlock funds
    Unlock { id: u64},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the lock info
    GetLock { id: u64 },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockResponse {
    pub owner: Addr,
    pub create: Timestamp,
    pub expire: Timestamp,
    pub funds: Vec<Coin>,
    pub complete: bool,
}
