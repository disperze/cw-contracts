use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Timestamp, WasmMsg,
};

use crate::balance::GenericBalance;
use crate::error::ContractError;
use crate::msg::{AllLocksResponse, ExecuteMsg, InstantiateMsg, LockInfo, QueryMsg, ReceiveMsg};
use crate::state::{Lock, State, LOCKS, STATE};

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
        ExecuteMsg::Lock { id, expire } => try_lock(
            deps,
            env,
            Balance::from(info.funds),
            &info.sender,
            id,
            expire,
        ),
        ExecuteMsg::IncreaseLock { id } => {
            try_increase_lock(deps, env, Balance::from(info.funds), &info.sender, id)
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
    id: String,
    expire: Timestamp,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    let current_time = env.block.time;
    if current_time.ge(&expire) {
        return Err(ContractError::LowExpired {});
    }

    let state = STATE.load(deps.storage)?;
    let diff = expire.minus_seconds(current_time.seconds());
    if diff.seconds().ge(&state.max_lock_time) {
        return Err(ContractError::HighExpired {});
    }

    let lock = Lock {
        create: env.block.time,
        expire,
        funds: balance.into(),
    };
    let key = (sender, id.to_owned());

    // try to store it, fail if the id was already in use
    LOCKS.update(deps.storage, key, |existing| match existing {
        None => Ok(lock),
        Some(_) => Err(ContractError::AlreadyInUse {}),
    })?;

    let res = Response::new()
        .add_attribute("action", "lock")
        .add_attribute("from", sender)
        .add_attribute("id", id);
    Ok(res)
}

pub fn try_increase_lock(
    deps: DepsMut,
    env: Env,
    balance: Balance,
    sender: &Addr,
    id: String,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    let key = (sender, id.to_owned());
    let mut lock = LOCKS.load(deps.storage, key.clone())?;

    if env.block.time.gt(&lock.expire) {
        return Err(ContractError::LockExpired {});
    }

    lock.funds.add_tokens(balance);
    LOCKS.save(deps.storage, key, &lock)?;

    let res = Response::new()
        .add_attribute("action", "increase_lock")
        .add_attribute("from", sender)
        .add_attribute("id", id);
    Ok(res)
}

pub fn try_unlock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let key = (&info.sender, id);
    let lock = LOCKS.load(deps.storage, key.clone())?;

    if env.block.time.le(&lock.expire) {
        return Err(ContractError::LockNotExpired {});
    }

    // unlock all tokens
    let messages = send_tokens(&info.sender, &lock.funds)?;

    // remove lock
    LOCKS.remove(deps.storage, key);

    let res = Response::new()
        .add_attribute("action", "unlock")
        .add_attribute("from", info.sender)
        .add_messages(messages);
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
    let sender = &api.addr_validate(&wrapper.sender)?;
    match msg {
        ReceiveMsg::Lock { id, expire } => try_lock(deps, env, balance, sender, id, expire),
        ReceiveMsg::IncreaseLock { id } => try_increase_lock(deps, env, balance, sender, id),
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
                funds: vec![],
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
        QueryMsg::Lock { address, id } => to_binary(&query_lock(deps, address, id)?),
        QueryMsg::AllLocks { address } => to_binary(&query_locks(deps, address)?),
    }
}

fn query_lock(deps: Deps, address: String, id: String) -> StdResult<LockInfo> {
    let key = (&deps.api.addr_validate(&address)?, id.to_owned());
    let lock = LOCKS.load(deps.storage, key)?;

    to_lock_info(lock, id)
}

fn query_locks(deps: Deps, address: String) -> StdResult<AllLocksResponse> {
    let owner_addr = &deps.api.addr_validate(&address)?;

    let locks_id: Result<Vec<_>, _> = LOCKS
        .prefix(owner_addr)
        .keys(deps.storage, None, None, Order::Ascending)
        .map(String::from_utf8)
        .collect();

    Ok(AllLocksResponse { locks: locks_id? })
}

fn to_lock_info(lock: Lock, id: String) -> StdResult<LockInfo> {
    // transform tokens
    let native_balance = lock.funds.native;
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

    let lock_info = LockInfo {
        id,
        create: lock.create,
        expire: lock.expire,
        native_balance,
        cw20_balance: cw20_balance?,
    };

    Ok(lock_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, from_binary, CosmosMsg, StdError, StdResult, SubMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

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
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // empty funds
        let info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Lock {
            id: "1".into(),
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
            id: "1".into(),
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
            id: "1".into(),
            expire: Timestamp::from_seconds(4000),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::HighExpired {}) => {}
            _ => panic!("Must return HighExpired error"),
        }

        // lock funds 1
        let msg = ExecuteMsg::Lock {
            id: "1".into(),
            expire: Timestamp::from_seconds(200),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // should exists lock
        let msg = QueryMsg::Lock {
            address: "anyone".into(),
            id: "1".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: LockInfo = from_binary(&res).unwrap();
        assert_eq!(0, value.create.seconds());
        assert_eq!(200, value.expire.seconds());

        // try lock same id
        let msg = ExecuteMsg::Lock {
            id: "1".into(),
            expire: Timestamp::from_seconds(200),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::AlreadyInUse {}) => {}
            _ => panic!("Must return AlreadyInUse error"),
        }

        // lock funds 2
        let msg = ExecuteMsg::Lock {
            id: "2".into(),
            expire: Timestamp::from_seconds(300),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // should exists lock
        let msg = QueryMsg::Lock {
            address: "anyone".into(),
            id: "2".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: LockInfo = from_binary(&res).unwrap();
        assert_eq!(300, value.expire.seconds());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AllLocks {
                address: "anyone".into(),
            },
        )
        .unwrap();
        let value: AllLocksResponse = from_binary(&res).unwrap();
        assert_eq!(2, value.locks.len())
    }

    #[test]
    fn increase_lock() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // lock funds
        let mut env = mock_env();
        let info = mock_info("anyone", &coins(2, "token"));
        env.block.time = Timestamp::from_seconds(100);
        let msg = ExecuteMsg::Lock {
            id: "1".into(),
            expire: Timestamp::from_seconds(200),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // try increase lock invalid id
        let info = mock_info("anyone", &coins(5, "token"));
        let msg = ExecuteMsg::IncreaseLock { id: "2".into() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::Std(StdError::NotFound { .. })) => {}
            _ => panic!("Must return StdError::NotFound error"),
        }

        // try increase lock after expire
        let info = mock_info("anyone", &coins(5, "token"));
        let msg = ExecuteMsg::IncreaseLock { id: "1".into() };
        env.block.time = Timestamp::from_seconds(201);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::LockExpired {}) => {}
            _ => panic!("Must return LockExpired error"),
        }

        // increase valid lock
        let msg = ExecuteMsg::IncreaseLock { id: "1".into() };
        env.block.time = Timestamp::from_seconds(120);
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // query funds lock
        let msg = QueryMsg::Lock {
            address: "anyone".into(),
            id: "1".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: LockInfo = from_binary(&res).unwrap();
        assert_eq!(coins(7, "token"), value.native_balance);
    }

    #[test]
    fn unlock() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

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
            id: "1".into(),
            expire: Timestamp::from_seconds(400),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // cannot unlock until expire
        let auth_info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Unlock { id: "1".into() };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let res = execute(deps.as_mut(), env.clone(), auth_info, msg);
        match res {
            Err(ContractError::LockNotExpired {}) => {}
            _ => panic!("Must return LockNotExpired error"),
        }

        // unlock funds
        let auth_info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Unlock { id: "1".into() };
        env.block.time = Timestamp::from_seconds(401);
        let res = execute(deps.as_mut(), env, auth_info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "anyone".into(),
                amount: coins(2, "token")
            }))
        );

        // should lock completed
        let msg = QueryMsg::Lock {
            address: "anyone".into(),
            id: "1".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg);

        match res {
            StdResult::Err(StdError::NotFound { .. }) => {}
            _ => panic!("Must return StdError::NotFound error"),
        }
    }
}
