use cosmwasm_std::{Coin, Timestamp};
use cw20::{Cw20Coin, Cw20ReceiveMsg};
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
    Lock { id: String, expire: Timestamp },
    /// Increase previous lock
    IncreaseLock { id: String },
    /// Unlock funds
    Unlock { id: String },
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Lock { id: String, expire: Timestamp },
    IncreaseLock { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the lock info
    Lock { address: String, id: String },
    /// Returns the locks by address
    AllLocks { address: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockInfo {
    pub id: String,
    pub create: Timestamp,
    pub expire: Timestamp,
    /// Funds in native tokens
    pub native_balance: Vec<Coin>,
    /// Funds in cw20 tokens
    pub cw20_balance: Vec<Cw20Coin>,
    pub complete: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct AllLocksResponse {
    pub locks: Vec<String>,
}
