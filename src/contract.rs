use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdResult, Timestamp, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, LockInfo, QueryMsg, ReceiveMsg};
use crate::state::{GenericBalance, Lock, State, LOCKS, STATE};
use cw2::set_contract_version;
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-lockbox";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        current: 0,
        max_lock_time: msg.max_lock_time,
        owner: info.sender,
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
        ExecuteMsg::Lock { expire } => {
            try_lock(deps, env, Balance::from(info.funds), &info.sender, expire)
        }
        ExecuteMsg::Unlock { id } => try_unlock(deps, env, info, id),
        ExecuteMsg::Receive(msg) => try_recive(deps, env, info, msg),
    }
}

pub fn try_lock(
    deps: DepsMut,
    env: Env,
    balance: Balance,
    sender: &Addr,
    expire: Timestamp,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    let current_time = env.block.time;
    if current_time.ge(&expire) {
        return Err(ContractError::LowExpired {});
    }

    let mut state = STATE.load(deps.storage)?;
    let diff = expire.minus_seconds(current_time.seconds());
    if diff.seconds().ge(&state.max_lock_time) {
        return Err(ContractError::HighExpired {});
    }

    let lock_data = Lock {
        create: env.block.time,
        expire,
        funds: balance.into(),
        complete: false,
        owner: sender.to_owned(),
    };

    state.current += 1;
    STATE.save(deps.storage, &state)?;
    LOCKS.save(deps.storage, &state.current.to_string(), &lock_data)?;

    Ok(Response {
        attributes: vec![
            attr("action", "lock"),
            attr("from", sender),
            attr("id", state.current),
        ],
        ..Response::default()
    })
}

pub fn try_unlock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let key = id.to_string();
    let mut lock = LOCKS.load(deps.storage, &key)?;
    if lock.complete {
        return Err(ContractError::LockComplete {});
    }

    if env.block.time.le(&lock.expire) {
        return Err(ContractError::LockNotExpired {});
    }

    lock.complete = true;
    LOCKS.save(deps.storage, &key, &lock)?;

    // unlock all tokens
    let messages = send_tokens(&info.sender, &lock.funds)?;

    let res = Response {
        messages,
        attributes: vec![attr("action", "unlock"), attr("from", info.sender)],
        ..Response::default()
    };

    Ok(res)
}

pub fn try_recive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender,
        amount: wrapper.amount,
    });
    let api = deps.api;
    match msg {
        ReceiveMsg::Lock { expire } => try_lock(
            deps,
            env,
            balance,
            &api.addr_validate(&wrapper.sender)?,
            expire,
        ),
    }
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        }
        .into()]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                send: vec![],
            };
            Ok(exec.into())
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLock { id } => to_binary(&query_lock(deps, id)?),
        QueryMsg::GetLocks { address } => {},
    }
}

fn query_lock(deps: Deps, id: u64) -> StdResult<LockInfo> {
    let lock = LOCKS.load(deps.storage, &id.to_string())?;
    // transform tokens
    let native_balance = lock.funds.native;
    // lock.funds.
    let cw20_balance: StdResult<Vec<_>> = lock
        .funds
        .cw20
        .into_iter()
        .map(|token| {
            Ok(Cw20Coin {
                address: token.address.into(),
                amount: token.amount,
            })
        })
        .collect();

    let details = LockInfo {
        owner: lock.owner,
        create: lock.create,
        expire: lock.expire,
        complete: lock.complete,
        native_balance,
        cw20_balance: cw20_balance?,
    };
    Ok(details)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, CosmosMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn lock() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // empty funds
        let info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(10),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::EmptyBalance {}) => {}
            _ => panic!("Must return EmptyBalance error"),
        }

        // lower expire
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(10),
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::LowExpired {}) => {}
            _ => panic!("Must return LowExpired error"),
        }

        // high expire
        env.block.time = Timestamp::from_seconds(0);
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(4000),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::HighExpired {}) => {}
            _ => panic!("Must return HighExpired error"),
        }

        // lock funds 1
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(200),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // should exists lock
        let msg = QueryMsg::GetLock { id: 1 };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: LockInfo = from_binary(&res).unwrap();
        assert_eq!(0, value.create.seconds());
        assert_eq!(200, value.expire.seconds());
        assert_eq!(false, value.complete);

        // lock funds 2
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(300),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // should exists lock
        let msg = QueryMsg::GetLock { id: 2 };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: LockInfo = from_binary(&res).unwrap();
        assert_eq!(300, value.expire.seconds());
        assert_eq!(false, value.complete);
    }

    #[test]
    fn unlock() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // lock funds
        let info = mock_info("anyone", &coins(2, "token"));
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(400),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // cannot unlock until expire
        let auth_info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Unlock { id: 1 };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let res = execute(deps.as_mut(), env.clone(), auth_info, msg);
        match res {
            Err(ContractError::LockNotExpired {}) => {}
            _ => panic!("Must return LockNotExpired error"),
        }

        // unlock funds
        let auth_info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Unlock { id: 1 };
        env.block.time = Timestamp::from_seconds(401);
        let res = execute(deps.as_mut(), env, auth_info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0],
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "anyone".into(),
                amount: coins(2, "token")
            })
        );

        // should lock completed
        let data = query(deps.as_ref(), mock_env(), QueryMsg::GetLock { id: 1 }).unwrap();

        let res: LockInfo = from_binary(&data).unwrap();
        assert_eq!(true, res.complete)
    }
}
