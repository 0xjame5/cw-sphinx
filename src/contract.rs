use std::ops::{Div, Mul, Range};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, OverflowError, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION, TOTAL_POOL_SIZE};
use crate::error::ContractError;
use crate::helpers::get_player_ranges;
use crate::models::PlayerRanges;
use crate::msg::{ExecuteMsg, InstantiateMsg, LotteryStateResponse, QueryMsg, TicketResponse};
use crate::state::{LotteryState, PlayerInfo, LOTTERY_STATE, PLAYERS, TICKET_UNIT_COST};
use crate::ContractError::PaymentError;

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

    // make sure ticket cost is greater than 0, or that it's a native
    // token. or apart of some predefined white list.
    let cost_per_ticket = msg.ticket_cost;
    TICKET_UNIT_COST.save(deps.storage, &cost_per_ticket)?;

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
        ExecuteMsg::ClaimTokens => execute_claim(deps, env, info),
    }
}

fn execute_buy_ticket(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bought_tickets: u64,
) -> Result<Response, ContractError> {
    // 1. if a user is trying to buy a ticket, before doing so.
    // 2. check to see if the lottery itself is expired, if so update state.
    let lottery_state = LOTTERY_STATE.load(deps.storage)?;

    match lottery_state {
        LotteryState::OPEN { expiration } => {
            if !(expiration.is_expired(&_env.block)) {
                // Take the amount of tokens sent, and verify its the amount needed.
                // Should be an exact amount.
                let ticket_cost = TICKET_UNIT_COST.load(deps.storage)?;
                // let bought_tickets_convert = u128::from(bought_tickets);
                // let bought_tickets_U128 = Uint128::new(bought_tickets_convert);

                // let cost = ticket_cost.amount.checked_mul(bought_tickets_U128);

                // let ex_cost = match cost {
                //     Ok(val) => Ok(val),
                //     Err(_) => Err(ContractError::PaymentError {}),
                // }?;

                // returns amount of denom wanted.
                // let amount_received_future = must_pay(&info, &ticket_cost.denom);
                //
                // let amount_rematch = match amount_received_future {
                //     Ok(val) => Ok(val),
                //     Err(_) => Err(ContractError::TicketBuyingNotEnoughFunds {}),
                // };

                // let amount_received_fut = amount_rematch?;

                // if ex_cost != amount_received_fut {
                //     Err(ContractError::TicketBuyingNotEnoughFunds {})
                // } else {
                // update_player(deps, &info, bought_tickets)?;
                Ok(Response::new())
                // }
            } else {
                // Lottery is expired, therefore go ahead and update the state of the contract
                // to next phase.
                LOTTERY_STATE.save(deps.storage, &LotteryState::CHOOSING {})?;
                Ok(Response::new())
            }
        }
        LotteryState::CHOOSING => Err(ContractError::TicketBuyingNotAvailable {}),
        LotteryState::CLOSED { .. } => Err(ContractError::TicketBuyingNotAvailable {}),
    }
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

fn execute_claim(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let lottery_state = LOTTERY_STATE.load(deps.storage)?;

    match lottery_state {
        LotteryState::CLOSED { winner, claimed } => {
            if !claimed {
                if info.sender == winner {
                    // send contract funds, and update lottery state to "closed and claimed"
                    Ok(Response::new())
                } else {
                    Err(ContractError::LotteryNotClaimedByCorrectUser {})
                }
            } else {
                Err(ContractError::LotteryAlreadyClaimed {})
            }
        }
        LotteryState::CHOOSING => Err(ContractError::LotteryNotClaimable {}),
        LotteryState::OPEN { .. } => Err(ContractError::LotteryNotClaimable {}),
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
    for player_range in player_ranges.ranges {
        if winner_ticket <= player_range.end_range && winner_ticket >= player_range.start_range {
            addr = Some(player_range.player_addr)
        }
    }

    let winner = addr.unwrap();

    LOTTERY_STATE.save(
        deps.storage,
        &LotteryState::CLOSED {
            winner,
            claimed: false,
        },
    )
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
    use cosmwasm_std::{coin, coins, Addr};

    use crate::msg::ExecuteMsg::BuyTicket;
    use crate::tests::common::{
        TestUser, TESTING_DURATION, TESTING_NATIVE_DENOM, TESTING_TICKET_COST,
    };

    use super::*;

    #[test]
    fn proper_initialization() {
        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM),
            lottery_duration: TESTING_DURATION,
        };

        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_message).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn buy_tickets() {
        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM),
            lottery_duration: TESTING_DURATION,
        };

        let mut deps = mock_dependencies();

        let _ = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(1000, "earth")),
            instantiate_message,
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

        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM),
            lottery_duration: TESTING_DURATION,
        };

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
            instantiate_message,
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
