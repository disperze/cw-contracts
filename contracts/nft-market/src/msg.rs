use cosmwasm_std::{Addr, Coin, Decimal};
use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fee: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Buy { offering_id: String },
    WithdrawNft { offering_id: String },
    ReceiveNft(Cw721ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SellNft {
    pub list_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCount {},
    GetOffer {
        contract: String,
        token_id: String,
    },
    GetOffers {
        seller: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all offers controlled by the contract.
    /// Return type: OffersResponse.
    AllOffers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct OffersResponse {
    pub offers: Vec<Offer>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Offer {
    pub id: String,
    pub token_id: String,
    pub contract: Addr,
    pub seller: Addr,
    pub list_price: Coin,
}
