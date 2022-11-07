use cosmwasm_schema::write_api;

use cw_disper_split::msg::{ExecuteMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
    }
}
