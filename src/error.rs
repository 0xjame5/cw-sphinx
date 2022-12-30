use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(cosmwasm_std::StdError),
    #[error("{0}")]
    OverFlowError(cosmwasm_std::OverflowError),

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

    #[error("Payment Error")]
    PaymentError {},
}

impl From<cw_utils::PaymentError> for ContractError {
    fn from(err: cw_utils::PaymentError) -> Self {
        match err {
            cw_utils::PaymentError::MissingDenom(_) => ContractError::PaymentError {},
            cw_utils::PaymentError::ExtraDenom(_) => ContractError::PaymentError {},
            cw_utils::PaymentError::MultipleDenoms { .. } => ContractError::PaymentError {},
            cw_utils::PaymentError::NoFunds { .. } => ContractError::PaymentError {},
            cw_utils::PaymentError::NonPayable { .. } => ContractError::PaymentError {},
        }
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
