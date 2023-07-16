use cosmwasm_std::Addr;

use crate::constants::MAX_HOUSE_FEE;
use crate::state::Config;
use crate::ContractError;
use crate::ContractError::Unauthorized;

pub fn is_admin(sender: Addr, config: Config) -> Result<(), ContractError> {
    if config.admin != sender {
        Err(Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_house_fee(house_fee: u64) -> Result<u64, ContractError> {
    if house_fee >= MAX_HOUSE_FEE {
        Err(ContractError::ContractInstantiationInvalidFee {})
    } else {
        Ok(house_fee)
    }
}
