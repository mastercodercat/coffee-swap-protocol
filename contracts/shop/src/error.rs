use cosmwasm_std::StdError;
use cosmwasm_std::Uint128;
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
    #[error("NotEnoughFunds")]
    NotEnoughFunds {},
    // for internal usage only !
    #[error("NotAnError")]
    NotAnError {},
}
