use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SellNft};
use crate::state::{get_fund, increment_offerings, Offering, State, OFFERINGS, STATE};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-dsp-nft-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        num_offerings: 0,
        owner: info.sender,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy { offering_id } => execute_buy(deps, info, offering_id),
        ExecuteMsg::WithdrawNft { offering_id } => execute_withdraw(deps, info, offering_id),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, info, msg),
    }
}

pub fn execute_buy(
    deps: DepsMut,
    info: MessageInfo,
    offering_id: String,
) -> Result<Response, ContractError> {
    // check if offering exists
    let off = OFFERINGS.load(deps.storage, &offering_id)?;

    // check for enough coins
    let off_fund = get_fund(info.funds.clone(), off.list_price.denom)?;
    if off_fund.amount < off.list_price.amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // create transfer msg
    let transfer_msg: CosmosMsg = BankMsg::Send {
        to_address: off.seller.clone().into(),
        amount: info.funds,
    }
    .into();

    // create transfer cw721 msg
    let cw721_transfer = Cw721ExecuteMsg::TransferNft {
        recipient: info.sender.clone().into(),
        token_id: off.token_id.clone(),
    };
    let cw721_transfer_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: off.contract.clone().into(),
        msg: to_binary(&cw721_transfer)?,
        send: vec![],
    }
    .into();

    //delete offering
    OFFERINGS.remove(deps.storage, &offering_id);

    let price_string = format!("{} {}", off_fund.amount, off_fund.denom);

    Ok(Response {
        messages: vec![transfer_msg, cw721_transfer_msg],
        attributes: vec![
            attr("action", "buy_nft"),
            attr("buyer", info.sender),
            attr("seller", off.seller),
            attr("paid_price", price_string),
            attr("token_id", off.token_id),
            attr("nft_contract", off.contract),
        ],
        ..Response::default()
    })
}

pub fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    offering_id: String,
) -> Result<Response, ContractError> {
    let off = OFFERINGS.load(deps.storage, &offering_id)?;
    if off.seller.ne(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
        recipient: off.seller.into(),
        token_id: off.token_id.clone(),
    };

    let exec_cw721_transfer: CosmosMsg = WasmMsg::Execute {
        contract_addr: off.contract.into(),
        msg: to_binary(&transfer_cw721_msg)?,
        send: vec![],
    }
    .into();

    OFFERINGS.remove(deps.storage, &offering_id);

    Ok(Response {
        messages: vec![exec_cw721_transfer],
        attributes: vec![
            attr("action", "withdraw_nft"),
            attr("seller", info.sender),
            attr("offering_id", offering_id),
        ],
        ..Response::default()
    })
}

pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: SellNft = match wrapper.msg {
        Some(bin) => Ok(from_binary(&bin)?),
        None => Err(ContractError::NoData {}),
    }?;

    // TODO: check if same token Id form same original contract is already on sale
    let id = increment_offerings(deps.storage)?.to_string();

    // save Offering
    let off = Offering {
        contract: info.sender.clone(),
        token_id: wrapper.token_id,
        seller: deps.api.addr_validate(&wrapper.sender)?,
        list_price: msg.list_price.clone(),
    };
    OFFERINGS.save(deps.storage, &id, &off)?;

    let price_string = format!("{} {}", msg.list_price.amount, msg.list_price.denom);
    Ok(Response {
        attributes: vec![
            attr("action", "sell_nft"),
            attr("nft_contract", info.sender),
            attr("seller", off.seller),
            attr("list_price", price_string),
            attr("token_id", off.token_id),
        ],
        ..Response::default()
    })
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse {
        count: state.num_offerings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
