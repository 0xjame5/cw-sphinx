use cosmwasm_std::{Addr, DepsMut};

use crate::constants::MAX_HOUSE_FEE;
use crate::state::ADMIN;
use crate::ContractError;
use crate::ContractError::Unauthorized;

pub fn validate_is_admin(sender: Addr, deps: &DepsMut) -> Result<Addr, ContractError> {
    let admin_addr = ADMIN.load(deps.storage)?;
    if admin_addr != sender {
        Err(Unauthorized {})
    } else {
        Ok(admin_addr)
    }
}

pub fn validate_house_fee(house_fee: u64) -> Result<u64, ContractError> {
    if house_fee >= MAX_HOUSE_FEE {
        Err(ContractError::ContractInstantiationInvalidFee {})
    } else {
        Ok(house_fee)
    }
}
