#[cfg(test)]
pub mod tests {
    use cosmwasm_std::Coin;
    use cw_utils::Duration;

    pub const TESTING_NATIVE_DENOM: &str = "ulotto";
    pub const TESTING_TICKET_COST: u128 = 1_000_u128;
    pub const TESTING_1_WEEK_IN_SECONDS: u64 = 604_800u64;
    pub const TESTING_DURATION: Duration = Duration::Time(TESTING_1_WEEK_IN_SECONDS);

    pub struct TestUser {
        pub addr: String,
        pub tickets: u64,
        pub coin: Coin,
    }

    pub const TEST_GOD: &str = "god"; // the original tester that owns ALL the coins
    pub const TEST_ADMIN: &str = "admin";
    pub const TEST_USER_1: &str = "user1";
    pub const TEST_USER_2: &str = "user2";
    pub const TEST_USER_3: &str = "user3";
}
