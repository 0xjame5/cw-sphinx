use cosmwasm_std::{Addr, DepsMut, Order, StdResult};

use crate::state::PLAYERS;

pub fn get_player_ranges<'a>(
    deps: &'a DepsMut,
) -> Box<dyn Iterator<Item = StdResult<(Addr, u64)>> + 'a> {
    PLAYERS.range(deps.storage, None, None, Order::Descending)
}
