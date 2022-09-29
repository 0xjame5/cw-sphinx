use std::ops::{Add, Div, Mul, Range};
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use rand_pcg::Pcg32;
use rand::{Rng, SeedableRng, rngs::StdRng, RngCore};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TicketResponse};
use crate::msg::QueryMsg::TicketCount;
use crate::helpers::get_player_ranges;
use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION, TOTAL_POOL_SIZE};
use crate::models::PlayerRanges;
use crate::state::{Config, CONFIG, PlayerInfo, PLAYERS};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config { cost_per_ticket: msg.ticket_cost };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyTicket { num_tickets } =>
            execute_buy_ticket(deps, env, info, num_tickets),
        // ExecuteAsAdmin with a random seed the value
        ExecuteMsg::ExecuteLottery { seed } => {
            execute_lottery(deps, env, info, seed)
        }
    }
}

fn execute_buy_ticket(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bought_tickets: u64,
) -> Result<Response, ContractError> {
    let some_player_info = PLAYERS.may_load(deps.storage, &info.sender)?;

    match some_player_info {
        None => {
            let new_player_info = PlayerInfo { tickets: bought_tickets };
            PLAYERS.save(deps.storage, &info.sender, &new_player_info)?
        }
        Some(player_info) => {
            let new_player_info = PlayerInfo { tickets: player_info.tickets + bought_tickets };
            PLAYERS.save(deps.storage, &info.sender, &new_player_info)?
        }
    }

    Ok(Response::new())
}

fn execute_lottery(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    seed: u64
) -> Result<Response, ContractError> {
    // TODO:: Update to make sure config admin to execute the lottery
    let _ = get_lottery_winner(deps, seed).unwrap();
    Ok(Response::new())
}

fn get_lottery_winner(deps: DepsMut, seed: u64) -> Option<Addr> {
    let mut rng = Pcg32::seed_from_u64(seed);

    let total_tickets = get_num_tickets(&deps);

    let winner_ticket = rng.gen_range(Range { start: 0, end: total_tickets });

    let player_ranges = create_player_ranges(&deps, total_tickets);

    let mut addr = None;
    for player_range in player_ranges.0 {
        if winner_ticket <= player_range.end_range && winner_ticket >= player_range.start_range {
            addr = Some(player_range.player_addr)
        }
    }
    addr
}

fn create_player_ranges(deps: &DepsMut, total_tickets: u64) -> PlayerRanges {
    let mut player_ranges = PlayerRanges::create();
    let mut current_index = 0;
    for player_result in get_player_ranges(deps) {
        let (addr, player_info) = player_result.unwrap();
        let number_of_tickets_to_ration = TOTAL_POOL_SIZE.div(total_tickets).mul(player_info.tickets);
        player_ranges.create_player_range(addr, current_index, current_index + number_of_tickets_to_ration);
        current_index += number_of_tickets_to_ration
    }
    player_ranges
}

fn get_num_tickets(deps: &DepsMut) -> u64 {
    let players = get_player_ranges(deps);
    let mut total_num_tickets: u64 = 0;
    for playersResult in players {
        let (_addr, player_info) = playersResult.unwrap();
        total_num_tickets += player_info.tickets
    }
    total_num_tickets
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        TicketCount { addr } =>
            to_binary(&query_ticket_count(deps, _env, addr)?),
    }
}


pub fn query_ticket_count(deps: Deps, _env: Env, addr: String) -> StdResult<TicketResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let res = PLAYERS.may_load(deps.storage, &addr)?;

    let tickets_opt: Option<u64> = match res {
        None => { None }
        Some(player_info) => { Some(player_info.tickets) }
    };

    Ok(TicketResponse { tickets: tickets_opt })
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use crate::msg::ExecuteMsg::BuyTicket;

    use super::*;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { ticket_cost: Default::default() };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn buy_tickets() {
        let mut deps = mock_dependencies();

        let ticket_cost = Uint128::from(1000_u32);
        let msg = InstantiateMsg { ticket_cost };

        let _ = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            msg)
            .unwrap();

        let _ = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            BuyTicket { num_tickets: 1 })
            .unwrap();

        let res = query_ticket_count(
            deps.as_ref(),
            mock_env(),
            "creator".to_string());

        assert!(res.is_ok());
        let ticket_response = res.unwrap();
        assert_eq!(ticket_response.tickets, Some(1))
    }
}
