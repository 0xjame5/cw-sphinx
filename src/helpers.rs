use cosmwasm_std::{Addr, CosmosMsg, DepsMut, Order, StdResult, to_binary, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::ExecuteMsg;
use crate::state::{PlayerInfo, PLAYERS};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}


pub fn get_player_ranges<'a>(deps: &'a DepsMut) -> Box<dyn Iterator<Item=StdResult<(cosmwasm_std::Addr, PlayerInfo)>> + 'a> {
    PLAYERS.range(deps.storage, None, None, Order::Descending)
}
