use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The ticket buying process right now is closed.")]
    TicketBuyingNotAvailable {},

    #[error("The current lottery is not executable because it is decided or still open.")]
    LotteryNotExecutable {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
