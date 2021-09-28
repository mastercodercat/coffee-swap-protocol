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

impl From<cw20_base::ContractError> for ContractError {
    fn from(err: cw20_base::ContractError) -> Self {
        match err {
            cw20_base::ContractError::Std(error) => ContractError::Std(error),
            cw20_base::ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw20_base::ContractError::NoAllowance {} => ContractError::NoAllowance {},
            cw20_base::ContractError::InvalidZeroAmount {} => ContractError::InvalidParam {},

            cw20_base::ContractError::CannotSetOwnAccount {}
            | cw20_base::ContractError::Expired {}
            | cw20_base::ContractError::CannotExceedCap {}

            // This should never happen, as this contract doesn't use logo
            | cw20_base::ContractError::LogoTooBig {}
            | cw20_base::ContractError::InvalidPngHeader {}
            | cw20_base::ContractError::InvalidXmlPreamble {} => {
                // ContractError::Std(StdError::generic_err(err.to_string()))
                ContractError::InternalError {}
            }
        }
    }
}
