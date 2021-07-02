use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub native_coin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Deposit token and get wrapped cw20 token
    Deposit {},
    /// Withdraw your wrapped token previous allowance
    Withdraw { amount: Uint128 },
    /// Only with the "owner" extension. Update cw20 smart-contract one time
    SetContract { contract: String },
    /// Only with "approval" extension. Receive tokens from cw20 contract and withdraw wrapped tokens.
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub cw20_contract: String,
    pub native_coin: String,
}
