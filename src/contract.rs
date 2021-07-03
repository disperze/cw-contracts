use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

use cw20::{AllowanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg, BalanceResponse};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender,
        contract: "".into(),
        native_coin: msg.native_coin,
        min_mint: msg.min_mint,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
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
        ExecuteMsg::Deposit {} => try_deposit(deps, env, info),
        ExecuteMsg::Withdraw { amount } => try_withdraw(deps, env, info, amount),
        ExecuteMsg::SetContract { contract } => try_update_contract(deps, info, contract),
        ExecuteMsg::Receive(cw20msg) => try_receive(deps, info, cw20msg),
    }
}

pub fn try_update_contract(
    deps: DepsMut,
    info: MessageInfo,
    contract: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    if !state.contract.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    deps.api.addr_validate(&contract)?;

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.contract = contract;
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn try_deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if info.funds.iter().any(|x| x.denom.ne(&state.native_coin)) {
        return Err(ContractError::InvalidCoin {});
    }

    let amount_to: Uint128 = info
        .funds
        .iter()
        .map(|x| x.amount)
        .fold(0u8.into(), |acc, amount| acc + amount);

    let balance = Cw20QueryMsg::Balance {
        address: env.contract.address.clone().into(),
    };

    let request = WasmQuery::Smart {
        contract_addr: state.contract.to_owned(),
        msg: to_binary(&balance)?,
    }
    .into();

    let res: BalanceResponse = deps.querier.query(&request)?;
    let mut msgs = vec![];
    if amount_to > res.balance {
        let mint_amount = amount_to.checked_sub(res.balance).unwrap();
        let cw20msg = Cw20ExecuteMsg::Mint {
            recipient: env.contract.address.into(),
            amount: if mint_amount > state.min_mint { mint_amount } else { state.min_mint },
        };
        let msg = WasmMsg::Execute {
            contract_addr: state.contract.to_owned(),
            msg: to_binary(&cw20msg)?,
            send: vec![],
        }
        .into();
        msgs.push(msg);
    }

    let cw20msg = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.clone().into(),
        amount: amount_to,
    };
    let msg = WasmMsg::Execute {
        contract_addr: state.contract,
        msg: to_binary(&cw20msg)?,
        send: vec![],
    }
    .into();
    msgs.push(msg);

    let attributes = vec![
        attr("action", "deposit"),
        attr("amount", amount_to),
        attr("sender", info.sender),
    ];
    Ok(Response {
        submessages: vec![],
        messages: msgs,
        attributes,
        data: None,
    })
}

pub fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    // transfer to contract cw20 tokens
    let transfer = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.clone().into(),
        recipient: env.contract.address.into(),
        amount,
    };

    let message = WasmMsg::Execute {
        contract_addr: state.contract.to_owned(),
        msg: to_binary(&transfer)?,
        send: vec![],
    }
    .into();

    // return native funds to user
    let bank_send = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![Coin::new(amount.into(), state.native_coin)],
    });

    Ok(Response {
        submessages: vec![],
        messages: vec![message, bank_send],
        attributes: vec![
            attr("action", "withdraw"),
            attr("amount", amount),
            attr("sender", info.sender),
        ],
        data: None,
    })
}

pub fn try_receive(
    deps: DepsMut,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // validate owner contract
    let state = STATE.load(deps.storage)?;
    if info.sender != state.contract {
        return Err(ContractError::Unauthorized {});
    }

    // send native coins to user
    let bank_send = CosmosMsg::Bank(BankMsg::Send {
        to_address: msg.sender.to_owned(),
        amount: vec![Coin::new(msg.amount.into(), state.native_coin)],
    });

    Ok(Response {
        submessages: vec![],
        messages: vec![bank_send],
        attributes: vec![
            attr("action", "receive_to_withdraw"),
            attr("amount", msg.amount),
            attr("sender", msg.sender),
        ],
        data: None,
    })
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_ctr_info(deps)?),
    }
}

pub fn query_ctr_info(deps: Deps) -> StdResult<InfoResponse> {
    let info = STATE.load(deps.storage)?;
    let res = InfoResponse {
        cw20_contract: info.contract,
        native_coin: info.native_coin,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{mock_dependencies_cw20_balance};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            native_coin: "inca".into(),
            min_mint: 10000u32.into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Info {}).unwrap();
        let value: InfoResponse = from_binary(&res).unwrap();
        assert_eq!("inca", value.native_coin);
        assert_eq!(true, value.cw20_contract.is_empty());
    }

    #[test]
    fn update_contract() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            min_mint: 10000u32.into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // set from anyone sender
        let info = mock_info("anyone", &[]);
        let cw20_contract: String = "juno145tr".into();
        let err = try_update_contract(deps.as_mut(), info, cw20_contract.to_owned()).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {:?}", e),
        }

        // set valid contract
        let info = mock_info("creator", &[]);
        let res = try_update_contract(deps.as_mut(), info, cw20_contract.to_owned()).unwrap();
        assert_eq!(0, res.messages.len());

        // try set new contract
        let info = mock_info("creator", &[]);
        let cw20_contract: String = "juno531tr".into();
        let err = try_update_contract(deps.as_mut(), info, cw20_contract.to_owned()).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn deposit() {
        let mut deps = mock_dependencies_cw20_balance(10u8.into());

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            min_mint: 5u8.into(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // set cw20 contract
        let info = mock_info("creator", &[]);
        let cw20_contract: String = "juno145tr".into();
        let res = try_update_contract(deps.as_mut(), info, cw20_contract.to_owned()).unwrap();
        assert_eq!(0, res.messages.len());

        // deposit invalid coin
        let env = mock_env();
        let info = mock_info("anyone", &coins(10, "btc"));
        let err = try_deposit(deps.as_mut(), env, info).unwrap_err();
        match err {
            ContractError::InvalidCoin {} => {}
            e => panic!("unexpected error: {:?}", e),
        }

        // valid coin
        let info = mock_info("creator", &coins(20, "juno"));
        let env = mock_env();
        let res = try_deposit(deps.as_mut(), env.clone(), info).unwrap();
        assert_eq!(res.messages.len(), 2);
        assert_eq!(
            res.messages[0],
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_contract,
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: env.contract.address.into(),
                    amount: 10u8.into(),
                })
                .unwrap(),
                send: vec![]
            })
        );
    }

    #[test]
    fn withdraw() {
        let mut deps = mock_dependencies(&[Coin::new(1000u32.into(), "juno")]);

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            min_mint: 10000u32.into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // set cw20 contract
        let info = mock_info("creator", &[]);
        let cw20_contract: String = "juno145tr".into();
        let res = try_update_contract(deps.as_mut(), info, cw20_contract.to_owned()).unwrap();
        assert_eq!(0, res.messages.len());

        // withdraw
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = try_withdraw(deps.as_mut(), env, info, 4u8.into()).unwrap();
        assert_eq!(2, res.messages.len());
    }

    #[test]
    fn receive() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            native_coin: "juno".into(),
            min_mint: 10000u32.into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // set cw20 contract
        let info = mock_info("creator", &[]);
        let cw20_contract: String = "cw20:contract".into();
        let res = try_update_contract(deps.as_mut(), info, cw20_contract.to_owned()).unwrap();
        assert_eq!(0, res.messages.len());

        // withdraw
        let info = mock_info("cw20:contract_anyone", &[]);
        let msg = Cw20ReceiveMsg {
            amount: 4u8.into(),
            sender: "owner".into(),
            msg: to_binary("").unwrap(),
        };

        let err = try_receive(deps.as_mut(), info, msg.clone()).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {:?}", e),
        }

        let info = mock_info("cw20:contract", &[]);
        let res = try_receive(deps.as_mut(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
