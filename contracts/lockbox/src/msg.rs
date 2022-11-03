use cosmwasm_std::{Coin, Timestamp};
use cw20::{Cw20Coin, Cw20ReceiveMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Max lock time in seconds
    pub max_lock_time: u64,
}

#[cw_serde]
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

#[cw_serde]
pub enum ReceiveMsg {
    Lock { id: String, expire: Timestamp },
    IncreaseLock { id: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the lock info
    #[returns(LockInfo)]
    Lock { address: String, id: String },
    /// Returns the locks by address
    #[returns(AllLocksResponse)]
    AllLocks { address: String },
}

#[cw_serde]
pub struct LockInfo {
    pub id: String,
    pub create: Timestamp,
    pub expire: Timestamp,
    /// Funds in native tokens
    pub native_balance: Vec<Coin>,
    /// Funds in cw20 tokens
    pub cw20_balance: Vec<Cw20Coin>,
}

#[cw_serde]
pub struct AllLocksResponse {
    pub locks: Vec<String>,
}
