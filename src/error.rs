use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Send some coins to lock funds")]
    EmptyBalance {},

    #[error("Expire time is lower")]
    LowExpired {},

    #[error("Expire time is higher")]
    HighExpired {},

    #[error("Lock has not expired")]
    LockNotExpired {},

    #[error("Lock has expired")]
    LockExpired {},

    #[error("Lock id already in use")]
    AlreadyInUse {},
}
