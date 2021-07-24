use cosmwasm_std::{
    attr, entry_point, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Timestamp,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, LockResponse, QueryMsg};
use crate::state::{State, LOCKS, STATE};

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
        ExecuteMsg::Lock { expire } => try_lock(deps, env, info, expire),
        ExecuteMsg::Unlock {} => try_unlock(deps, env, info),
    }
}

pub fn try_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    expire: Timestamp,
) -> Result<Response, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    let current_time = env.block.time;
    if current_time.ge(&expire) {
        return Err(ContractError::Unauthorized {});
    }

    let state = STATE.load(deps.storage)?;
    let diff = expire.minus_seconds(current_time.seconds());
    if diff.seconds().ge(&state.max_lock_time) {
        return Err(ContractError::Unauthorized {});
    }

    let key = info.sender.clone();
    let lock = LOCKS.may_load(deps.storage, &key)?;
    if let Some(..) = lock {
        return Err(ContractError::Unauthorized {});
    }

    let lock_data = LockResponse {
        start: env.block.time,
        end: expire,
        amount: info.funds,
    };
    LOCKS.save(deps.storage, &key, &lock_data)?;

    Ok(Response {
        attributes: vec![
            attr("action", "lock"),
            attr("from", info.sender),
            // attr("amount", lock_data.amount.into()),
        ],
        ..Response::default()
    })
}

pub fn try_unlock(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let key = info.sender.clone();
    let lock = LOCKS.load(deps.storage, &key)?;
    if env.block.time.le(&lock.end) {
        return Err(ContractError::Unauthorized {});
    }

    let bank_send = BankMsg::Send {
        amount: lock.amount.clone(),
        to_address: info.sender.clone().into(),
    }
    .into();

    let res = Response {
        messages: vec![bank_send],
        attributes: vec![
            attr("action", "unlock"),
            attr("from", info.sender),
            // attr("amount", lock.amount),
        ],
        ..Response::default()
    };

    LOCKS.remove(deps.storage, &key);
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLock { address } => to_binary(&query_lock(deps, address)?),
    }
}

fn query_lock(deps: Deps, address: String) -> StdResult<LockResponse> {
    let sender_addr = deps.api.addr_validate(&address)?;
    let lock = LOCKS.load(deps.storage, &sender_addr)?;

    Ok(lock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, StdError, StdResult};

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
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // max time
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(4000),
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // lock funds
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(200),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // should exists lock
        let msg = QueryMsg::GetLock {
            address: "anyone".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: LockResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.start.seconds());
        assert_eq!(200, value.end.seconds());
    }

    #[test]
    fn unlock() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            max_lock_time: 3600,
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // lock funds
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Lock {
            expire: Timestamp::from_seconds(400),
        };
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // beneficiary can release it
        let auth_info = mock_info("anyone", &[]);
        let msg = ExecuteMsg::Unlock {};
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let res = execute(deps.as_mut(), env.clone(), auth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Unlock {};
        env.block.time = Timestamp::from_seconds(401);
        let _res = execute(deps.as_mut(), env, auth_info, msg).unwrap();

        // should no exist lock
        let msg = QueryMsg::GetLock {
            address: "anyone".into(),
        };
        let res = query(deps.as_ref(), mock_env(), msg);
        match res {
            StdResult::Err(StdError::NotFound { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }
}
