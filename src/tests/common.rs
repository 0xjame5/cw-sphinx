use crate::msg::InstantiateMsg;
use cosmwasm_std::Uint128;
use cw_utils::Duration;

pub const TESTING_TICKET_COST: Uint128 = Uint128::new(1_000_000u128);
pub const TESTING_1_WEEK_IN_SECONDS: u64 = 604_800u64;
pub const TESTING_DURATION: Duration = Duration::Time(TESTING_1_WEEK_IN_SECONDS);
pub const TESTING_INST_MSG: InstantiateMsg = InstantiateMsg {
    ticket_cost: TESTING_TICKET_COST,
    lottery_duration: TESTING_DURATION,
};

pub struct TestUser {
    pub addr: String,
    pub tickets: u64,
}
