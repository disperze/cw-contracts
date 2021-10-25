use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("No {denom} tokens sent")]
    EmptyBalance { denom: String },

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Logo binary data exceeds 5KB limit")]
    LogoTooBig {},

    #[error("Invalid xml preamble for SVG")]
    InvalidXmlPreamble {},

    #[error("Invalid png header")]
    InvalidPngHeader {},
}

impl From<Cw20ContractError> for ContractError {
    fn from(err: Cw20ContractError) -> Self {
        match err {
            Cw20ContractError::Std(error) => ContractError::Std(error),
            Cw20ContractError::Unauthorized {} => ContractError::Unauthorized {},
            Cw20ContractError::CannotSetOwnAccount {} => ContractError::CannotSetOwnAccount {},
            Cw20ContractError::InvalidZeroAmount {} => ContractError::InvalidZeroAmount {},
            Cw20ContractError::Expired {} => ContractError::Expired {},
            Cw20ContractError::NoAllowance {} => ContractError::NoAllowance {},
            Cw20ContractError::CannotExceedCap {} => ContractError::CannotExceedCap {},
            Cw20ContractError::LogoTooBig {} => ContractError::LogoTooBig {},
            Cw20ContractError::InvalidXmlPreamble {} => ContractError::InvalidXmlPreamble {},
            Cw20ContractError::InvalidPngHeader {} => ContractError::InvalidPngHeader {},
        }
    }
}
