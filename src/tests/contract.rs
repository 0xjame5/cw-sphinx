use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, coins, Addr};
use crate::contract::{execute, instantiate, query_ticket_count};

use crate::msg::ExecuteMsg::BuyTicket;
use crate::msg::InstantiateMsg;
use crate::tests::common::{TestUser, TESTING_DURATION, TESTING_NATIVE_DENOM, TESTING_TICKET_COST};

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
        mock_info("creator", &coins(1000, TESTING_NATIVE_DENOM)),
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
            coin: coin(TESTING_TICKET_COST * 1, TESTING_NATIVE_DENOM),
        },
        TestUser {
            addr: "testUserA".to_string(),
            tickets: 10,
            coin: coin(TESTING_TICKET_COST * 10, TESTING_NATIVE_DENOM),
        },
        TestUser {
            addr: "testUserB".to_string(),
            tickets: 10,
            coin: coin(TESTING_TICKET_COST * 10, TESTING_NATIVE_DENOM),
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
            mock_info(&test_user.addr, &[test_user.coin]),
            BuyTicket {
                num_tickets: test_user.tickets,
            },
        )
        .unwrap();

        let res = query_ticket_count(deps.as_ref(), mock_env(), Addr::unchecked(test_user.addr));
        assert!(res.is_ok());

        let ticket_response = res.unwrap();
        assert_eq!(ticket_response.tickets, Some(test_user.tickets));
    }
}
