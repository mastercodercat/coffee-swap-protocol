use cosmwasm_std::{StdError, StdResult, Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::products::{CoffeeCup, IngredientPortion};
use std::ops::Add;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
     pub coffee_token_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetPrice {
        coffee_shop_key: String,
        id: Uint128,
        price: Uint128,
    },
    BuyCoffee {
        coffee_shop_key: String,
        id: Uint128,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Owner {},
    Menu { coffee_shop_key: String },
    // Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance { address: String },
}

impl InstantiateMsg {
    pub fn validate(&self) -> StdResult<()> {
        // Check addr

        Ok(())
    }
}