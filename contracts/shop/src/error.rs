use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("InvalidParam")]
    InvalidParam {},
    #[error("NotEnoughIngredients")]
    NotEnoughIngredients {},
    #[error("InternalError")]
    InternalError {},
    // not implemented or not used errors
    #[error("NotEnoughFunds")]
    NotEnoughFunds {},
    #[error("NotEnoughFunds")]
    NoAllowance {},
    // for internal usage only !
    #[error("NotAnError")]
    NotAnError {},
}