use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not enough funds passed for the number of tickets being bought.")]
    TicketBuyingNotEnoughFunds {},

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
