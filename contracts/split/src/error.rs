use cosmwasm_std::{Decimal, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Min 2 users are required")]
    MinUsers {},

    #[error("Duplicate users")]
    DuplicateUsers {},

    #[error("Empty Balance")]
    EmptyBalance {},

    #[error("Invalid math calc")]
    MathCalc {},

    #[error("Invalid total {total} percentage")]
    InvalidPercentage { total: Decimal },
}
