#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Empty, StdError, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

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
    fn buy_tickets_and_lottery() {
        let mut app = mock_app();
        let lotto_code_id = app.store_code(contract_lotto());

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
