use cosmwasm_std::{Addr, Decimal};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub native_coin: String,
    pub users: Vec<User>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Split {},
}

#[cw_serde]
pub struct User {
    pub address: Addr,
    pub percent: Decimal,
}
