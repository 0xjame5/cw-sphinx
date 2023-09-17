#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Empty, Uint128};

    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use cw_utils::Duration;

    use crate::msg::{ExecuteMsg, InstantiateMsg, LotteryStateResponse, QueryMsg, TicketResponse};
    use crate::state::LotteryState;
    use crate::test_util::tests::{
        TESTING_DURATION, TESTING_NATIVE_DENOM, TESTING_TICKET_COST, TEST_ADMIN, TEST_GOD,
        TEST_USER_1, TEST_USER_2, TEST_USER_3,
    };
    use crate::ContractError;

    fn expire(voting_period: Duration) -> impl Fn(&mut BlockInfo) {
        move |block: &mut BlockInfo| {
            match voting_period {
                Duration::Time(duration) => block.time = block.time.plus_seconds(duration + 1),
                Duration::Height(duration) => block.height += duration + 1,
            };
        }
    }

    fn mock_app(owner: Addr, coins: Vec<Coin>) -> App {
        App::new(|router, _, storage| {
            // initialization moved to App construction
            router.bank.init_balance(storage, &owner, coins).unwrap()
        })
    }

    pub fn contract_lotto() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    #[test]
    fn instantiate_buy_1_ticket_and_execute() {
        let (mut app, lotto_code_id) = setup_app();

        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM),
            lottery_duration: TESTING_DURATION,
            admin: TEST_ADMIN.to_string(),
            house_fee: 500, // 5%
        };

        let lotto_contract_addr = app
            .instantiate_contract(
                lotto_code_id,
                Addr::unchecked(TEST_ADMIN),
                &instantiate_message,
                &[],
                "yolo",
                None,
            )
            .unwrap();

        let _buy_ticket_response_1 = app
            .execute_contract(
                Addr::unchecked(TEST_USER_1),
                lotto_contract_addr.clone(),
                &ExecuteMsg::BuyTicket { num_tickets: 1 },
                &[Coin {
                    denom: TESTING_NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1_000u128),
                }],
            )
            .unwrap();
        // Validate user only bought 1 ticket and balance is now reflecting now
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_1), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(4_000u128)
            }
        );

        app.update_block(expire(TESTING_DURATION));

        // Note that this would be empty, shit would return OK.
        // This is because this would update the contract to next state. However,
        // the next call would fail.
        let _buy_ticket_response_2 = app
            .execute_contract(
                Addr::unchecked(TEST_USER_2),
                lotto_contract_addr.clone(),
                &ExecuteMsg::BuyTicket { num_tickets: 1 },
                &[],
            )
            .unwrap();

        let ticket_response_for_user_1: TicketResponse = app
            .wrap()
            .query_wasm_smart(
                lotto_contract_addr.clone(),
                &QueryMsg::TicketCount {
                    addr: Addr::unchecked(TEST_USER_1),
                },
            )
            .unwrap();

        assert_eq!(
            ticket_response_for_user_1,
            TicketResponse { tickets: Some(1) }
        );

        let ticket_response_for_user_2: TicketResponse = app
            .wrap()
            .query_wasm_smart(
                lotto_contract_addr.clone(),
                &QueryMsg::TicketCount {
                    addr: Addr::unchecked(TEST_USER_2),
                },
            )
            .unwrap();

        assert_eq!(ticket_response_for_user_2, TicketResponse { tickets: None });

        // Below validate user cannot buy tickets once the state has changed
        let app_resp_err = app
            .execute_contract(
                Addr::unchecked(TEST_USER_3),
                lotto_contract_addr.clone(),
                &ExecuteMsg::BuyTicket { num_tickets: 1 },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::TicketBuyingNotAvailable {},
            app_resp_err.downcast().unwrap()
        );

        let _claim_resp = app
            .execute_contract(
                Addr::unchecked(TEST_ADMIN),
                lotto_contract_addr.clone(),
                &ExecuteMsg::ExecuteLottery { seed: 12 },
                &[],
            )
            .unwrap();

        let query_resp: LotteryStateResponse = app
            .wrap()
            .query_wasm_smart(lotto_contract_addr.clone(), &QueryMsg::LotteryState {})
            .unwrap();

        assert_eq!(
            query_resp,
            LotteryStateResponse {
                lotto_state: LotteryState::CLOSED {
                    winner: (Addr::unchecked(TEST_USER_1)),
                    claimed: false
                },
                total_tickets: 1
            }
        );

        let _claim_response = app
            .execute_contract(
                Addr::unchecked(TEST_USER_1),
                lotto_contract_addr.clone(),
                &ExecuteMsg::ClaimTokens {},
                &[],
            )
            .unwrap();

        let query_resp_post_claim: LotteryStateResponse = app
            .wrap()
            .query_wasm_smart(lotto_contract_addr.clone(), &QueryMsg::LotteryState {})
            .unwrap();

        assert_eq!(
            query_resp_post_claim,
            LotteryStateResponse {
                lotto_state: LotteryState::CLOSED {
                    winner: (Addr::unchecked(TEST_USER_1)),
                    claimed: true
                },
                total_tickets: 1
            }
        );

        let contract_balance = app
            .wrap()
            .query_balance(lotto_contract_addr, TESTING_NATIVE_DENOM)
            .unwrap();

        assert_eq!(
            contract_balance,
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Default::default(),
            }
        );

        // User should have owned minus fees
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_1), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(4_950u128)
            }
        );

        // Admin should have some more tokens now
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_ADMIN), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(10_050u128)
            }
        );
    }

    #[test]
    fn instantiate_buy_with_two_players_ticket_and_execute() {
        let (mut app, lotto_code_id) = setup_app();

        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM),
            lottery_duration: TESTING_DURATION,
            admin: TEST_ADMIN.to_string(),
            house_fee: 500, // 5%
        };

        let lotto_contract_addr = app
            .instantiate_contract(
                lotto_code_id,
                Addr::unchecked(TEST_ADMIN),
                &instantiate_message,
                &[],
                "yolo",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked(TEST_USER_1),
            lotto_contract_addr.clone(),
            &ExecuteMsg::BuyTicket { num_tickets: 1 },
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(1_000u128),
            }],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked(TEST_USER_1),
            lotto_contract_addr.clone(),
            &ExecuteMsg::BuyTicket { num_tickets: 2 },
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(2_000u128),
            }],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked(TEST_USER_2),
            lotto_contract_addr.clone(),
            &ExecuteMsg::BuyTicket { num_tickets: 3 },
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(3_000u128),
            }],
        )
        .unwrap();

        // Both players balances should've updated.
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_1), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(2_000u128)
            }
        );
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_2), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(2_000u128)
            }
        );

        app.update_block(expire(TESTING_DURATION));

        // Note that this would be empty, shit would return OK.
        // This is because this would update the contract to next state. However,
        // the next call would fail.
        app.execute_contract(
            Addr::unchecked(TEST_USER_2),
            lotto_contract_addr.clone(),
            &ExecuteMsg::BuyTicket { num_tickets: 1 },
            &[],
        )
        .unwrap();

        // Validate user 1 and 2 bought 3 tickets only. User 3 bought none.
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<TicketResponse>(
                    lotto_contract_addr.clone(),
                    &QueryMsg::TicketCount {
                        addr: Addr::unchecked(TEST_USER_1),
                    },
                )
                .unwrap(),
            TicketResponse { tickets: Some(3) }
        );
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<TicketResponse>(
                    lotto_contract_addr.clone(),
                    &QueryMsg::TicketCount {
                        addr: Addr::unchecked(TEST_USER_2),
                    },
                )
                .unwrap(),
            TicketResponse { tickets: Some(3) }
        );
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<TicketResponse>(
                    lotto_contract_addr.clone(),
                    &QueryMsg::TicketCount {
                        addr: Addr::unchecked(TEST_USER_3),
                    },
                )
                .unwrap(),
            TicketResponse { tickets: None }
        );

        // Below validate user cannot buy tickets once the state has changed
        assert_eq!(
            app.execute_contract(
                Addr::unchecked(TEST_USER_3),
                lotto_contract_addr.clone(),
                &ExecuteMsg::BuyTicket { num_tickets: 1 },
                &[],
            )
            .unwrap_err()
            .downcast::<ContractError>()
            .unwrap(),
            ContractError::TicketBuyingNotAvailable {},
        );

        // Let admin move lottery forward.
        app.execute_contract(
            Addr::unchecked(TEST_ADMIN),
            lotto_contract_addr.clone(),
            &ExecuteMsg::ExecuteLottery { seed: 12 },
            &[],
        )
        .unwrap();

        // Lottery state response shows expected state
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<LotteryStateResponse>(
                    lotto_contract_addr.clone(),
                    &QueryMsg::LotteryState {}
                )
                .unwrap(),
            LotteryStateResponse {
                lotto_state: LotteryState::CLOSED {
                    winner: (Addr::unchecked(TEST_USER_2)),
                    claimed: false
                },
                total_tickets: 6
            }
        );

        // Let user 1 try and claim tokens claim tokens and get error
        assert_eq!(
            app.execute_contract(
                Addr::unchecked(TEST_USER_1),
                lotto_contract_addr.clone(),
                &ExecuteMsg::ClaimTokens {},
                &[],
            )
            .unwrap_err()
            .downcast::<ContractError>()
            .unwrap(),
            ContractError::LotteryNotClaimedByCorrectUser {}
        );

        assert_eq!(
            app.wrap()
                .query_wasm_smart::<LotteryStateResponse>(
                    lotto_contract_addr.clone(),
                    &QueryMsg::LotteryState {}
                )
                .unwrap(),
            LotteryStateResponse {
                lotto_state: LotteryState::CLOSED {
                    winner: (Addr::unchecked(TEST_USER_2)),
                    claimed: false
                },
                total_tickets: 6
            }
        );

        // Let user 2 claim tokens
        app.execute_contract(
            Addr::unchecked(TEST_USER_2),
            lotto_contract_addr.clone(),
            &ExecuteMsg::ClaimTokens {},
            &[],
        )
        .unwrap();

        // Validate total tickets, lottery state is closed with claim and winner set.
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<LotteryStateResponse>(
                    lotto_contract_addr.clone(),
                    &QueryMsg::LotteryState {}
                )
                .unwrap(),
            LotteryStateResponse {
                lotto_state: LotteryState::CLOSED {
                    winner: (Addr::unchecked(TEST_USER_2)),
                    claimed: true
                },
                total_tickets: 6
            }
        );

        // Validate contract has no remaining balance and has been flushed out.
        assert_eq!(
            app.wrap()
                .query_balance(lotto_contract_addr, TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Default::default(),
            }
        );

        // User 2 lost all that was put in.
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_1), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(2_000u128)
            }
        );

        // User 3 won minus the fees
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_2), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(7_700u128)
            }
        );

        // Admin should have some more tokens now from fee
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_ADMIN), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(10_300u128)
            }
        );
    }

    fn setup_app() -> (App, u64) {
        // God is genesis, the whole defined sentient
        let mut app = mock_app(
            Addr::unchecked(TEST_GOD),
            vec![Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(100_000_000_000u128),
            }],
        );
        // God is sending token admin, user1 and user 2
        app.send_tokens(
            Addr::unchecked(TEST_GOD),
            Addr::unchecked(TEST_ADMIN),
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(10_000u128),
            }],
        )
        .unwrap();
        app.send_tokens(
            Addr::unchecked(TEST_GOD),
            Addr::unchecked(TEST_USER_1),
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(5_000u128),
            }],
        )
        .unwrap();
        app.send_tokens(
            Addr::unchecked(TEST_GOD),
            Addr::unchecked(TEST_USER_2),
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(5_000u128),
            }],
        )
        .unwrap();

        let lotto_code_id = app.store_code(contract_lotto());
        (app, lotto_code_id)
    }
}
