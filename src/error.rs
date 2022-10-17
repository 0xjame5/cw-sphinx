use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The current lottery is not executable because it is decided or still open.")]
    LotteryNotExecutable {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
