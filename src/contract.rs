use std::borrow::Borrow;
use std::ops::{Add, Div, Mul, Range};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::{Duration, Expiration};
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use rand_pcg::Pcg32;

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION, TOTAL_POOL_SIZE};
use crate::error::ContractError;
use crate::helpers::get_player_ranges;
use crate::models::PlayerRanges;
use crate::msg::{ExecuteMsg, InstantiateMsg, LotteryStateResponse, QueryMsg, TicketResponse};
use crate::state::{Config, LotteryState, PlayerInfo, CONFIG, LOTTERY_STATE, PLAYERS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        cost_per_ticket: msg.ticket_cost,
    };
    CONFIG.save(deps.storage, &config)?;

    LOTTERY_STATE.save(
        deps.storage,
        &LotteryState::OPEN {
            expiration: msg.lottery_duration.after(&_env.block),
        },
    )?;

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
        ExecuteMsg::BuyTicket { num_tickets } => execute_buy_ticket(deps, env, info, num_tickets),
        ExecuteMsg::ExecuteLottery { seed } => execute_lottery(deps, env, info, seed),
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
            let new_player_info = PlayerInfo {
                tickets: bought_tickets,
            };
            PLAYERS.save(deps.storage, &info.sender, &new_player_info)?
        }
        Some(player_info) => {
            let new_player_info = PlayerInfo {
                tickets: player_info.tickets + bought_tickets,
            };
            PLAYERS.save(deps.storage, &info.sender, &new_player_info)?
        }
    }
    Ok(Response::new())
}

fn execute_lottery(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    seed: u64,
) -> Result<Response, ContractError> {
    let lottery_state = LOTTERY_STATE.load(deps.storage)?;
    match lottery_state {
        LotteryState::CHOOSING => choose_winner(deps, seed)?,
        LotteryState::OPEN { .. } => {
            //
        } // not met minimum number of tickets
        LotteryState::CLOSED { winner: _winner } => {}
    }
    // TODO:: Update to make sure config admin to execute the lottery

    Ok(Response::new())
}

fn choose_winner(deps: DepsMut, seed: u64) -> StdResult<()> {
    let mut rng = Pcg32::seed_from_u64(seed);
    let total_tickets = get_num_tickets(&deps);
    let winner_ticket = rng.gen_range(Range {
        start: 0,
        end: total_tickets,
    });
    let player_ranges = create_player_ranges(&deps, total_tickets);

    let mut addr = None;
    for player_range in player_ranges.0 {
        if winner_ticket <= player_range.end_range && winner_ticket >= player_range.start_range {
            addr = Some(player_range.player_addr)
        }
    }

    let winner = addr.unwrap();

    LOTTERY_STATE.save(deps.storage, &LotteryState::CLOSED { winner })
}

fn create_player_ranges(deps: &DepsMut, total_tickets: u64) -> PlayerRanges {
    let mut player_ranges = PlayerRanges::create();
    let mut current_index = 0;
    for player_result in get_player_ranges(deps) {
        let (addr, player_info) = player_result.unwrap();
        let number_of_tickets_to_ration =
            TOTAL_POOL_SIZE.div(total_tickets).mul(player_info.tickets);
        player_ranges.create_player_range(
            addr,
            current_index,
            current_index + number_of_tickets_to_ration,
        );
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
        QueryMsg::TicketCount { addr } => to_binary(&query_ticket_count(deps, _env, addr)?),
        QueryMsg::LotteryState => to_binary(&query_lottery_state(deps, _env)?),
    }
}

pub fn query_lottery_state(deps: Deps, _env: Env) -> StdResult<LotteryStateResponse> {
    let lottery_state = LOTTERY_STATE.load(deps.storage)?;
    Ok(LotteryStateResponse {
        lotto_state: lottery_state,
    })
}

pub fn query_ticket_count(deps: Deps, _env: Env, addr: String) -> StdResult<TicketResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let res = PLAYERS.may_load(deps.storage, &addr)?;

    let tickets_opt: Option<u64> = match res {
        None => None,
        Some(player_info) => Some(player_info.tickets),
    };

    Ok(TicketResponse {
        tickets: tickets_opt,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Uint128};

    use crate::msg::ExecuteMsg::{BuyTicket, ExecuteLottery};

    use super::*;

    pub const TESTING_TICKET_COST: Uint128 = Uint128::new(1_000_000u128);
    pub const TESTING_DURATION: Duration = Duration::Time(604_800);
    pub const TESTING_INST_MSG: InstantiateMsg = InstantiateMsg {
        ticket_cost: TESTING_TICKET_COST,
        lottery_duration: TESTING_DURATION,
    };

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, TESTING_INST_MSG.clone()).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn buy_tickets() {
        let mut deps = mock_dependencies();

        let _ = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            TESTING_INST_MSG,
        )
        .unwrap();

        let _ = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            BuyTicket { num_tickets: 1 },
        )
        .unwrap();

        let res = query_ticket_count(deps.as_ref(), mock_env(), "creator".to_string());

        assert!(res.is_ok());
        let ticket_response = res.unwrap();
        assert_eq!(ticket_response.tickets, Some(1))
    }

    #[test]
    fn buy_tickets_and_lottery() {
        let mut deps = mock_dependencies();

        let _ = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            TESTING_INST_MSG,
        )
        .unwrap();

        struct TestUser {
            pub addr: String,
            pub tickets: u64,
        }

        let mut test_users = vec![];
        test_users.push(TestUser {
            addr: "creator".to_string(),
            tickets: 1,
        });
        test_users.push(TestUser {
            addr: "a".to_string(),
            tickets: 10,
        });
        test_users.push(TestUser {
            addr: "b".to_string(),
            tickets: 10,
        });
        test_users.push(TestUser {
            addr: "c".to_string(),
            tickets: 10,
        });

        for test_user in test_users {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(&test_user.addr, &coins(1000, "earth")),
                BuyTicket {
                    num_tickets: test_user.tickets,
                },
            )
            .unwrap();
        }

        let res = query_ticket_count(deps.as_ref(), mock_env(), "creator".to_string());

        assert!(res.is_ok());
        let ticket_response = res.unwrap();
        assert_eq!(ticket_response.tickets, Some(1));

        /*
        - When we buy and run the rando lottery we should have the ability to decide a winner,
          and in this case we should have 1 winner.
        */
        let _ = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            ExecuteLottery { seed: 124212 },
        );
    }
}
