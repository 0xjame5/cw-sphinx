use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PLAYERS: Map<&Addr, PlayerInfo> = Map::new("players");
pub const CONFIG: Item<Config> = Item::new("config");
pub const LOTTERY_STATE: Item<LotteryState> = Item::new("lotto_state");
pub const TOTAL_TICKETS: Item<u64> = Item::new("total_tickets");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    // the native cost per ticket. 1 juno for 1 ticket, or ..
    pub cost_per_ticket: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum LotteryState {
    OPEN {
        // the period at which the lottery will stop executing open tickets. the goal is
        // to have a time window for buying
        expiration: Expiration,
    },
    CHOOSING,
    CLOSED {
        winner: Addr,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerInfo {
    pub tickets: u64,
}
