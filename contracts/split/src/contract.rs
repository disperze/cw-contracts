use crate::error::ContractError;

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{State, UserParams, STATE, USERS};
use crate::utils::has_unique_elements;
use cosmwasm_std::{
    entry_point, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
};
use std::ops::{Add, Mul};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.users.is_empty() || msg.users.len() < 2 {
        return Err(ContractError::MinUsers {});
    }

    if !has_unique_elements(msg.users.clone().into_iter().map(|u| u.address)) {
        return Err(ContractError::DuplicateUsers {});
    }

    let total = msg
        .users
        .iter()
        .map(|f| f.percent)
        .reduce(|a, b| a.add(b))
        .unwrap();

    if !total.eq(&Decimal::one()) {
        return Err(ContractError::InvalidPercentage { total });
    }

    for user in msg.users {
        let params = UserParams {
            percent: user.percent,
            split: Uint128::zero(),
        };
        USERS.save(deps.storage, &user.address, &params)?;
    }

    let state = State {
        native_coin: msg.native_coin,
        total_split: Uint128::zero(),
        owner: info.sender,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Split {} => try_split(deps, env, info),
    }
}

pub fn try_split(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    let balance = deps.querier.query_balance(
        env.contract.address.to_string(),
        state.native_coin.to_owned(),
    )?;
    if balance.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    let mut user = USERS.load(deps.storage, &info.sender)?;
    let to_send = user
        .percent
        .mul(balance.amount.add(state.total_split))
        .checked_sub(user.split)
        .map_err(|_| ContractError::MathCalc {})?;

    if to_send.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    user.split += to_send;
    state.total_split += to_send;
    USERS.save(deps.storage, &info.sender, &user)?;
    STATE.save(deps.storage, &state)?;

    let bank_send = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![Coin::new(to_send.into(), state.native_coin)],
    });

    let res = Response::new()
        .add_attribute("action", "split")
        .add_attribute("amount", to_send)
        .add_attribute("sender", info.sender)
        .add_message(bank_send);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::User;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coin, coins, Addr, Decimal, SubMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let users = vec![
            User {
                address: Addr::unchecked("user1"),
                percent: Decimal::percent(15),
            },
            User {
                address: Addr::unchecked("user1"),
                percent: Decimal::percent(85),
            },
        ];
        let msg = InstantiateMsg {
            native_coin: "dev".into(),
            users,
        };
        let info = mock_info("creator", &[]);
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::DuplicateUsers { .. }) => {}
            _ => panic!("Must return DuplicateUsers error"),
        }

        let mut users = vec![
            User {
                address: Addr::unchecked("user1"),
                percent: Decimal::percent(15),
            },
            User {
                address: Addr::unchecked("user2"),
                percent: Decimal::percent(25),
            },
            User {
                address: Addr::unchecked("user3"),
                percent: Decimal::percent(50),
            },
        ];
        let msg = InstantiateMsg {
            native_coin: "dev".into(),
            users: users.clone(),
        };
        let info = mock_info("creator", &[]);
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);
        match res {
            Err(ContractError::InvalidPercentage { .. }) => {}
            _ => panic!("Must return InvalidPercentage error"),
        }

        users.push(User {
            address: Addr::unchecked("user4"),
            percent: Decimal::percent(10),
        });
        let msg = InstantiateMsg {
            native_coin: "dev".into(),
            users,
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn split() {
        let balance = [coin(100, "dev")];
        let mut deps = mock_dependencies_with_balance(&balance);
        let users = vec![
            User {
                address: Addr::unchecked("user1"),
                percent: Decimal::percent(30),
            },
            User {
                address: Addr::unchecked("user2"),
                percent: Decimal::percent(20),
            },
            User {
                address: Addr::unchecked("user3"),
                percent: Decimal::percent(50),
            },
        ];
        let info = mock_info("creator", &[]);
        let msg = InstantiateMsg {
            native_coin: "dev".into(),
            users: users.clone(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("user1", &[]);
        let msg = ExecuteMsg::Split {};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".into(),
                amount: coins(30, "dev")
            }))
        );
    }
}
