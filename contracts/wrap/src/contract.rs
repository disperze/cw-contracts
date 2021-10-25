use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_minter,
    query_token_info,
};
use cw20_base::enumerable::{query_all_accounts, query_all_allowances};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-wjuno";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::new(0),
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    let state = State {
        owner: info.sender,
        native_coin: msg.native_coin,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => try_deposit(deps, env, info),
        ExecuteMsg::Withdraw { amount } => try_withdraw(deps, env, info, amount),

        // cw20 standard
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Burn { amount } => Ok(execute_burn(deps, env, info, amount)?),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::BurnFrom { owner, amount } => {
            Ok(execute_burn_from(deps, env, info, owner, amount)?)
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
    }
}

pub fn try_deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let deposit = info
        .funds
        .iter()
        .find(|x| x.denom == state.native_coin)
        .ok_or(ContractError::EmptyBalance {
            denom: state.native_coin,
        })?;

    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    execute_mint(
        deps,
        env,
        sub_info,
        info.sender.clone().into(),
        deposit.amount,
    )?;

    let res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("amount", deposit.amount)
        .add_attribute("sender", info.sender);
    Ok(res)
}

pub fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    execute_burn(deps, env, info.clone(), amount)?;

    // return native coin
    let bank_send = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![Coin::new(amount.into(), state.native_coin)],
    });

    let res = Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount)
        .add_attribute("sender", info.sender)
        .add_message(bank_send);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // cw20 standard
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&query_all_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coin, coins, from_binary, SubMsg};
    use cw20::BalanceResponse;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            name: "wjuno".into(),
            decimals: 6.into(),
            symbol: "WJUNO".into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn deposit() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            name: "wjuno".into(),
            decimals: 6.into(),
            symbol: "WJUNO".into(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // deposit invalid coin
        let env = mock_env();
        let info = mock_info("anyone", &coins(10, "btc"));
        let err = try_deposit(deps.as_mut(), env, info).unwrap_err();
        match err {
            ContractError::EmptyBalance { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }

        // valid coin
        let info = mock_info("creator", &coins(20, "juno"));
        let env = mock_env();
        let res = try_deposit(deps.as_mut(), env.clone(), info).unwrap();
        assert_eq!(res.messages.len(), 0);

        // check balance query
        let data = query(
            deps.as_ref(),
            env,
            QueryMsg::Balance {
                address: String::from("creator"),
            },
        )
        .unwrap();
        let response: BalanceResponse = from_binary(&data).unwrap();
        assert_eq!(response.balance, 20u8.into());
    }

    #[test]
    fn withdraw() {
        let mut deps = mock_dependencies_with_balance(&coins(1000u32.into(), "juno"));

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            name: "wjuno".into(),
            decimals: 6.into(),
            symbol: "WJUNO".into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // deposit
        let env = mock_env();
        let amount_deposit = 5u8;
        let info = mock_info("creator", &coins(amount_deposit.into(), "juno"));
        let res = try_deposit(deps.as_mut(), env.clone(), info).unwrap();
        assert_eq!(res.messages.len(), 0);

        // withdraw
        let info = mock_info("creator", &[]);
        let amount_withdraw = 4u8;
        let res = try_withdraw(deps.as_mut(), env.clone(), info, amount_withdraw.into()).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                amount: vec![coin(amount_withdraw.into(), "juno")],
                to_address: "creator".into(),
            }))
        );

        // check balance query
        let data = query(
            deps.as_ref(),
            env,
            QueryMsg::Balance {
                address: String::from("creator"),
            },
        )
        .unwrap();
        let response: BalanceResponse = from_binary(&data).unwrap();
        assert_eq!(response.balance, (amount_deposit - amount_withdraw).into());
    }
}
