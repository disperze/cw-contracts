use crate::cw721::Cw721ReceiveMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub fee: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    Buy {
        offering_id: String,
    },
    WithdrawNft {
        offering_id: String,
    },
    ReceiveNft(Cw721ReceiveMsg),
    /// only admin.
    WithdrawFees {
        amount: Uint128,
        denom: String,
    },
    /// only admin.
    ChangeFee {
        fee: Decimal,
    },
}

#[cw_serde]
pub struct SellNft {
    pub list_price: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CountResponse)]
    GetCount {},
    #[returns(FeeResponse)]
    GetFee {},
    /// With Enumerable extension.
    /// Requires pagination. Lists all offers controlled by the contract.
    /// Return type: OffersResponse.
    #[returns(OffersResponse)]
    AllOffers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct CountResponse {
    pub count: u64,
}

#[cw_serde]
pub struct FeeResponse {
    pub fee: Decimal,
}

#[cw_serde]
pub struct OffersResponse {
    pub offers: Vec<Offer>,
}

#[cw_serde]
pub struct Offer {
    pub id: String,
    pub token_id: String,
    pub contract: Addr,
    pub seller: Addr,
    pub list_price: Coin,
}
