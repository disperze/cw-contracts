#![cfg(test)]

use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{to_binary, OwnedDeps, Querier, QuerierResult, SystemResult, Uint128};
use cw20::{AllowanceResponse, Expiration};

pub fn mock_dependencies_allowance(allowance: Uint128) -> OwnedDeps<MockStorage, MockApi, MyMockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MyMockQuerier { amount: allowance },
    }
}

pub struct MyMockQuerier {
    amount: Uint128,
}

impl Querier for MyMockQuerier {
    fn raw_query(&self, _: &[u8]) -> QuerierResult {
        let allowance_res = AllowanceResponse {
            allowance: self.amount,
            expires: Expiration::Never {},
        };

        SystemResult::Ok(to_binary(&allowance_res).into())
    }
}
