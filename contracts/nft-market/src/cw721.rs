use cosmwasm_schema::cw_serde;

use cosmwasm_std::Binary;

#[cw_serde]
pub enum Cw721ExecuteMsg {
    TransferNft { recipient: String, token_id: String },
}

#[cw_serde]
pub struct Cw721ReceiveMsg {
    pub sender: String,
    pub token_id: String,
    pub msg: Binary,
}
