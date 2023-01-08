#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Empty, Uint128};

    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use cw_utils::Duration;

    use crate::msg::{ExecuteMsg, InstantiateMsg, LotteryStateResponse, QueryMsg};
    use crate::state::LotteryState;
    use crate::test_util::tests::{
        TESTING_DURATION, TESTING_NATIVE_DENOM, TESTING_TICKET_COST, TEST_ADMIN, TEST_USER_1,
        TEST_USER_2, TEST_USER_3,
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
    fn instantiate_buy_tickets_and_execute() {
        let mut app = mock_app(
            Addr::unchecked(TEST_ADMIN),
            vec![Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(100_000_000_000u128),
            }],
        );

        app.send_tokens(
            Addr::unchecked(TEST_ADMIN),
            Addr::unchecked(TEST_USER_1),
            &[Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(100_000u128),
            }],
        )
        .unwrap();

        let lotto_code_id = app.store_code(contract_lotto());

        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM),
            lottery_duration: TESTING_DURATION,
            admin: Addr::unchecked(TEST_ADMIN),
            house_fee: 500,
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

        // User should have 99k
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_1), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(99_000u128)
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
                }
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
                }
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

        // User should have original amount, 100k
        assert_eq!(
            app.wrap()
                .query_balance(Addr::unchecked(TEST_USER_1), TESTING_NATIVE_DENOM)
                .unwrap(),
            Coin {
                denom: TESTING_NATIVE_DENOM.to_string(),
                amount: Uint128::new(100_000u128)
            }
        );
    }
}
