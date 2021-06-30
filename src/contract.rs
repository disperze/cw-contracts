use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    Uint128, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{State, STATE};

use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

const JUNO_COIN: &str = "ujuno";

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender,
        contract: None,
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
        ExecuteMsg::Deposit {} => try_deposit(deps, info),
        ExecuteMsg::Withdraw { amount } => try_withdraw(deps, env, info, amount),
        ExecuteMsg::SetContract { contract } => try_update_contract(deps, info, contract),
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

    if state.contract.is_some() {
        return Err(ContractError::Unauthorized {});
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.contract = Some(contract);
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn try_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    if info.funds.iter().any(|x| x.denom.ne(JUNO_COIN)) {
        return Err(ContractError::Unauthorized {});
    }

    let amount_to = info
        .funds
        .iter()
        .map(|x| x.amount)
        .fold(0u8.into(), |acc, amount| acc + amount);
    let mint = Cw20ExecuteMsg::Mint {
        recipient: info.sender.clone().into(),
        amount: amount_to,
    };

    let state = STATE.load(deps.storage)?;
    let message = WasmMsg::Execute {
        contract_addr: state.contract.unwrap(),
        msg: to_binary(&mint)?,
        send: vec![],
    }
    .into();

    let attributes = vec![
        attr("action", "deposit"),
        attr("amount", amount_to),
        attr("sender", info.sender),
    ];
    Ok(Response {
        submessages: vec![],
        messages: vec![message],
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
    // check balance
    let balance = Cw20QueryMsg::Balance {
        address: info.sender.clone().into(),
    };

    let state = STATE.load(deps.storage)?;
    let request = WasmQuery::Smart {
        contract_addr: state.contract.clone().unwrap(),
        msg: to_binary(&balance)?,
    }
    .into();
    let res: BalanceResponse = deps.querier.query(&request)?;

    if res.balance > amount {
        return Err(ContractError::Unauthorized {});
    }

    // burn msg
    let burn = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.clone().into(),
        recipient: env.contract.address.into(),
        amount
    };

    let message = WasmMsg::Execute {
        contract_addr: state.contract.unwrap(),
        msg: to_binary(&burn)?,
        send: vec![],
    }
    .into();

    // return funds
    let bank_send = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![Coin::new(amount.into(), JUNO_COIN)],
    });

    // return msg
    let attributes = vec![
        attr("action", "withdraw"),
        attr("amount", amount),
        attr("sender", info.sender),
    ];

    Ok(Response {
        submessages: vec![],
        messages: vec![message, bank_send],
        attributes,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(17, value.count);
    }
}
