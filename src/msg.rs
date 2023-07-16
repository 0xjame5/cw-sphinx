use crate::state::{Config, LotteryState};
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Coin};
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    pub ticket_cost: Coin,
    pub lottery_duration: Duration,
    pub admin: String,
    pub house_fee: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    BuyTicket { num_tickets: u64 },
    ExecuteLottery { seed: u64 },
    ClaimTokens {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TicketResponse)]
    TicketCount { addr: Addr },
    #[returns(LotteryStateResponse)]
    LotteryState {},
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct TicketResponse {
    pub tickets: Option<u64>,
}

#[cw_serde]
pub struct LotteryStateResponse {
    pub lotto_state: LotteryState,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
