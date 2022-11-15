#[cfg(test)]
mod tests {
    use crate::msg::ExecuteMsg;
    use cosmwasm_std::{Addr, BlockInfo, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use cw_utils::Duration;

    fn expire(voting_period: Duration) -> impl Fn(&mut BlockInfo) {
        move |block: &mut BlockInfo| {
            match voting_period {
                Duration::Time(duration) => block.time = block.time.plus_seconds(duration + 1),
                Duration::Height(duration) => block.height += duration + 1,
            };
        }
    }

    use crate::tests::common::{TESTING_DURATION, TESTING_INST_MSG};

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

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

        let lotto_contract_addr = app
            .instantiate_contract(
                lotto_code_id,
                Addr::unchecked(ADMIN),
                &TESTING_INST_MSG,
                &[],
                "yolo",
                None,
            )
            .unwrap();

        app.update_block(expire(TESTING_DURATION));

        let buy_ticket_exec_msg = ExecuteMsg::BuyTicket { num_tickets: 1 };

        let app_response_1 = app
            .execute_contract(
                Addr::unchecked("TEST_USER_1"),
                lotto_contract_addr.clone(),
                &buy_ticket_exec_msg,
                &[],
            )
            .unwrap();

        let app_response_2 = app
            .execute_contract(
                Addr::unchecked("TEST_USER_2"),
                lotto_contract_addr.clone(),
                &buy_ticket_exec_msg,
                &[],
            )
            .unwrap();

        // how do i add a time dilation, such that we can update the stupid as contract.
        // let resp = execute(
        //     deps.as_mut(),
        //     mock_env(),
        //     mock_info("creator", &coins(1000, "earth")),
        //     ExecuteLottery { seed: 124212 },
        // );

        // having app and then adding time dilation is the only way. fuck LOL
    }
}
