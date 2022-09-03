#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TicketResponse};
use crate::msg::QueryMsg::TicketCount;
use crate::state::{Config, CONFIG, PlayerInfo, PLAYERS};

const CONTRACT_NAME: &str = "crates.io:cw-lootboxes";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // TODO, instnatiate Map of Players, store information like how much it costs a ticket
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        cost_per_ticket: 0
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyTicket { num_tickets } =>
            execute_buy_ticket(deps, _env, info, num_tickets),
        // ExecuteAsAdmin with a random seed the value
    }
}

fn execute_buy_ticket(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bought_tickets: i32
) -> Result<Response, ContractError> {
    // let sender_addr_str = info.sender.to_string();

    let cfg = PLAYERS.may_load(deps.storage, &info.sender)?;

    match cfg {
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


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TicketCount { addr } =>
            to_binary(&query_ticket_count(deps, _env, addr)?),
    }
}


pub fn query_ticket_count(deps: Deps, _env: Env, addr: String) -> StdResult<TicketResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let res = PLAYERS.may_load(deps.storage, &addr)?;

    let tickets_opt: Option<i32> = match res {
        None => { None }
        Some(player_info) => { Some(player_info.tickets) }
    };

    Ok(TicketResponse { tickets: tickets_opt })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

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
    fn increment() {
        let mut deps = mock_dependencies();
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

    }
}
