use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ADMIN: Item<Addr> = Item::new("admin_addr");

pub const HOUSE_FEE: Item<Decimal> = Item::new("house_fee");

// Map of players and their ticket allocation
pub const PLAYERS: Map<&Addr, PlayerInfo> = Map::new("players");

// The cost per ticket. can be native token, juno or what have you.
pub const TICKET_UNIT_COST: Item<Coin> = Item::new("ticket_cost");

// Current state of the ongoing lottery
pub const LOTTERY_STATE: Item<LotteryState> = Item::new("lotto_state");

// Cheap way of grabbing all the total number of tickets.
// Better way is just Other choice is
pub const TOTAL_TICKETS: Item<u64> = Item::new("total_tickets");

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
        claimed: bool,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerInfo {
    pub tickets: u64,
}
