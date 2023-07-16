use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub house_fee: Decimal,
    pub ticket_unit_cost: Coin, // The cost per ticket. can be native token, juno or what have you.
}

pub const CONFIG: Item<Config> = Item::new("config");

// Map of players and their ticket allocation
pub const PLAYERS: Map<Addr, u64> = Map::new("players");

// Current state of the ongoing lottery
pub const LOTTERY_STATE: Item<LotteryState> = Item::new("lotto_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum LotteryState {
    OPEN {
        // the period at which the lottery will stop executing open tickets. the goal is
        // to have a time window for buying
        expiration: Expiration,
    },
    CHOOSING {},
    CLOSED {
        winner: Addr,
        claimed: bool,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerInfo {
    pub tickets: u64,
}
