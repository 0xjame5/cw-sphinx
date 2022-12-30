use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(cosmwasm_std::StdError),

    #[error("{0}")]
    OverFlowError(cosmwasm_std::OverflowError),

    #[error("{0}")]
    PaymentError(cw_utils::PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not enough funds passed for the number of tickets being bought.")]
    TicketBuyingIncorrectAmount {},

    #[error("The ticket buying process right now is closed.")]
    TicketBuyingNotAvailable {},

    #[error("The current lottery is not executable because it is decided or still open.")]
    LotteryNotExecutable {},

    #[error("The current lottery is not in a state that rewards can be claimed.")]
    LotteryNotClaimable {},

    #[error("The current lottery winner has already claimed earnings")]
    LotteryAlreadyClaimed {},

    #[error("The current lottery winner has already claimed earnings")]
    LotteryNotClaimedByCorrectUser {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}

impl From<cw_utils::PaymentError> for ContractError {
    fn from(err: cw_utils::PaymentError) -> Self {
        ContractError::PaymentError(err)
    }
}

impl From<cosmwasm_std::StdError> for ContractError {
    fn from(err: cosmwasm_std::StdError) -> Self {
        ContractError::Std(err)
    }
}

impl From<cosmwasm_std::OverflowError> for ContractError {
    fn from(err: cosmwasm_std::OverflowError) -> Self {
        ContractError::OverFlowError(err)
    }
}
