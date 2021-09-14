use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Order, Response, StdResult, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{
    CountResponse, ExecuteMsg, InstantiateMsg, Offer, OffersResponse, QueryMsg, SellNft,
};
use crate::state::{get_fund, increment_offerings, maybe_addr, Offering, State, OFFERINGS, STATE};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use cw_storage_plus::Bound;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-dsp-nft-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

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
        QueryMsg::GetOffer { contract, token_id } => {
            to_binary(&query_token_id(deps, contract, token_id)?)
        }
        QueryMsg::GetOffers {
            seller,
            start_after,
            limit,
        } => to_binary(&query_tokens(deps, seller, start_after, limit)?),
        QueryMsg::AllOffers { start_after, limit } => {
            to_binary(&query_all(deps, start_after, limit)?)
        }
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse {
        count: state.num_offerings,
    })
}
fn query_all(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<OffersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let offers: StdResult<Vec<Offer>> = OFFERINGS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(map_offer))
        .collect();

    Ok(OffersResponse { offers: offers? })
}

fn query_tokens(
    deps: Deps,
    seller: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<OffersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));
    let address = deps.api.addr_validate(&seller)?;

    let offers: StdResult<Vec<Offer>> = OFFERINGS
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|item| match item {
            Ok((_, v)) => v.seller == address,
            Err(..) => false,
        })
        .take(limit)
        .map(|item| item.map(map_offer))
        .collect();

    Ok(OffersResponse { offers: offers? })
}

fn query_token_id(deps: Deps, contract: String, token_id: String) -> StdResult<OffersResponse> {
    let offers: StdResult<Vec<Offer>> = OFFERINGS
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|item| match item {
            Ok((_, v)) => v.contract == contract && v.token_id == token_id,
            Err(..) => false,
        })
        .map(|item| item.map(map_offer))
        .collect();

    Ok(OffersResponse { offers: offers? })
}

fn map_offer((k, v): (Vec<u8>, Offering)) -> Offer {
    Offer {
        id: String::from_utf8_lossy(&k).to_string(),
        token_id: v.token_id,
        contract: v.contract,
        seller: v.seller,
        list_price: v.list_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coin;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    fn setup(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn sell_nft() {
        let mut deps = mock_dependencies(&[]);
        setup(deps.as_mut());

        let sell_msg = SellNft {
            list_price: coin(1000, "earth"),
        };

        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            token_id: "1".into(),
            sender: "owner".into(),
            msg: Some(to_binary(&sell_msg).unwrap()),
        });
        let info = mock_info("nft-collectibles", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        assert_eq!(0, res.messages.len());

        let msg = QueryMsg::GetOffer {
            contract: "nft-collectibles".into(),
            token_id: "1".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: OffersResponse = from_binary(&res).unwrap();

        assert_eq!("1", value.offers.first().unwrap().token_id);

        let msg = QueryMsg::GetOffers {
            seller: "owner".into(),
            limit: None,
            start_after: None,
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: OffersResponse = from_binary(&res).unwrap();

        assert_eq!(1, value.offers.len());
    }
}
