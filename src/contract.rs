use std::ops::{Div, Mul, Range};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_utils::Duration;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION, TOTAL_POOL_SIZE};
use crate::error::ContractError;
use crate::helpers::get_player_ranges;
use crate::models::PlayerRanges;
use crate::msg::{ExecuteMsg, InstantiateMsg, LotteryStateResponse, QueryMsg, TicketResponse};
use crate::state::{Config, LotteryState, PlayerInfo, CONFIG, LOTTERY_STATE, PLAYERS};

/*
Each individual contract owner will be able to creat their own ticket cost. We require it to be
set to be more defined.
*/
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

/*
There are three states of the lottery.
- Open - we are actively allowing users to keep adding to the lottery state
- Choosing - we no longer allow a user to vote, however we have
- Closed - the winner of the lottery is stored in this state, and we return it alongside the address

After choosing a closed vote, a winner should be able to then execute a function on the contract
to retrieve their assets. 1% of the rewards will be set to the DAO treasury for continued deving.
""
*/
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
    // if a user is trying to buy a ticket, before doing so. check to see if the

    update_player(deps, &info, bought_tickets)?;

    Ok(Response::new())
}

fn update_player(deps: DepsMut, info: &MessageInfo, bought_tickets: u64) -> StdResult<()> {
    let some_player_info = PLAYERS.may_load(deps.storage, &info.sender)?;
    match some_player_info {
        None => {
            let new_player_info = PlayerInfo {
                tickets: bought_tickets,
            };
            PLAYERS.save(deps.storage, &info.sender, &new_player_info)
        }
        Some(player_info) => {
            let new_player_info = PlayerInfo {
                tickets: player_info.tickets + bought_tickets,
            };
            PLAYERS.save(deps.storage, &info.sender, &new_player_info)
        }
    }
}

fn execute_lottery(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    seed: u64,
) -> Result<Response, ContractError> {
    let lottery_state = LOTTERY_STATE.load(deps.storage)?;
    match lottery_state {
        LotteryState::CHOOSING => {
            // TODO(james):: Check before choosing winner to see if the caller is whitelisted (admin).
            choose_winner(deps, seed)?;
            Ok(Response::new())
        }
        LotteryState::OPEN { .. } => Err(ContractError::LotteryNotExecutable {}),
        LotteryState::CLOSED { .. } => Err(ContractError::LotteryNotExecutable {}),
    }
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
    for player_results in players {
        let (_addr, player_info) = player_results.unwrap();
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

pub fn query_ticket_count(deps: Deps, _env: Env, addr: Addr) -> StdResult<TicketResponse> {
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
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw_multi_test::App;

    use crate::msg::ExecuteMsg::{BuyTicket, ExecuteLottery};
    use crate::tests::common::{TestUser, TESTING_INST_MSG};

    use super::*;

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

        let res = query_ticket_count(
            deps.as_ref(),
            mock_env(),
            Addr::unchecked("creator".to_string()),
        );

        assert!(res.is_ok());
        let ticket_response = res.unwrap();
        assert_eq!(ticket_response.tickets, Some(1))
    }

    #[test]
    fn buy_multiple_tickets() {
        let mut deps = mock_dependencies();
        let test_users = vec![
            TestUser {
                addr: "creator".to_string(),
                tickets: 1,
            },
            TestUser {
                addr: "testUserA".to_string(),
                tickets: 10,
            },
            TestUser {
                addr: "testUserB".to_string(),
                tickets: 10,
            },
        ];

        let _ = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            TESTING_INST_MSG,
        )
        .unwrap();

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

            let res =
                query_ticket_count(deps.as_ref(), mock_env(), Addr::unchecked(test_user.addr));
            assert!(res.is_ok());

            let ticket_response = res.unwrap();
            assert_eq!(ticket_response.tickets, Some(test_user.tickets));
        }
    }
}
