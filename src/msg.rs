use cosmwasm_std::{Coin, Timestamp};
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
    Unlock {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the lock info
    GetLock { address: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockResponse {
    pub start: Timestamp,
    pub end: Timestamp,
    /// Amount locked in native tokens
    pub amount: Vec<Coin>,
}
