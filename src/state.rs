use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};



pub const PLAYERS: Map<&Addr, PlayerInfo> = Map::new("players");
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub cost_per_ticket: Uint128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerInfo { pub tickets: u64 }
