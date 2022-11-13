use cosmwasm_std::{to_binary, Addr, CosmosMsg, DepsMut, Order, StdResult, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::ExecuteMsg;
use crate::state::{PlayerInfo, PLAYERS};

pub fn get_player_ranges<'a>(
    deps: &'a DepsMut,
) -> Box<dyn Iterator<Item = StdResult<(Addr, PlayerInfo)>> + 'a> {
    PLAYERS.range(deps.storage, None, None, Order::Descending)
}
