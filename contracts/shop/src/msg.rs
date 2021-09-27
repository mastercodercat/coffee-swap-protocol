use std::ops::Add;

use cosmwasm_std::{Addr, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::products::{CoffeeCup, IngredientPortion};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token_addr: Addr,
    pub shop_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    LoadIngredients {
        coffee_shop_key: String,
        portions: Vec<IngredientPortion>,
    },
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
    Balance {},
    Price {
        coffee_shop_key: String,
        id: Uint128,
    },
    Menu {
        coffee_shop_key: String,
    },
    Recipes {
        coffee_shop_key: String,
    },
    Ingredients {
        coffee_shop_key: String,
    },
    // Implements CW20. Returns the current balance of the given address, 0 if unset.
    CW20Balance {
        address: String,
    },
}
