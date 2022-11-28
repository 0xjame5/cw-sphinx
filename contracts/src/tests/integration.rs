#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, BlockInfo, coin, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use cw_utils::Duration;

    use crate::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::tests::common::{TESTING_DURATION, TESTING_NATIVE_DENOM, TESTING_TICKET_COST, ADMIN};

    fn expire(voting_period: Duration) -> impl Fn(&mut BlockInfo) {
        move |block: &mut BlockInfo| {
            match voting_period {
                Duration::Time(duration) => block.time = block.time.plus_seconds(duration + 1),
                Duration::Height(duration) => block.height += duration + 1,
            };
        }
    }




    fn mock_app() -> App {
        App::default()
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
        let mut app = mock_app();
        let lotto_code_id = app.store_code(contract_lotto());

        let instantiate_message = InstantiateMsg {
            ticket_cost: coin(TESTING_TICKET_COST, TESTING_NATIVE_DENOM ),
            lottery_duration: TESTING_DURATION,
        };

        let lotto_contract_addr = app
            .instantiate_contract(
                lotto_code_id,
                Addr::unchecked(ADMIN),
                &instantiate_message,
                &[],
                "yolo",
                None,
            )
            .unwrap();

        let buy_ticket_exec_msg = ExecuteMsg::BuyTicket { num_tickets: 1 };

        let app_response_1 = app
            .execute_contract(
                Addr::unchecked("TEST_USER_1"),
                lotto_contract_addr.clone(),
                &buy_ticket_exec_msg,
                &[],
            )
            .unwrap();

        app.update_block(expire(TESTING_DURATION));

        // Note that this would be empty, shit would return OK.
        // This is because this would update the contract to next state. However,
        // the next call would fail.
        let app_response_2 = app
            .execute_contract(
                Addr::unchecked("TEST_USER_2"),
                lotto_contract_addr.clone(),
                &buy_ticket_exec_msg,
                &[],
            )
            .unwrap();

        let app_resp_err = app
            .execute_contract(
                Addr::unchecked("TEST_USER_3"),
                lotto_contract_addr.clone(),
                &buy_ticket_exec_msg,
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::TicketBuyingNotAvailable {},
            app_resp_err.downcast().unwrap()
        );
    }
}
