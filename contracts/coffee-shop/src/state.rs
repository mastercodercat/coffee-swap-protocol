use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::products;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub balance: Uint128,
    pub minted_amount: Uint128,

    // pub recipes: Vec<products::CoffeeRecipe>,
    pub menu: Vec<products::CoffeeCup>,
}

pub const STATE: Item<State> = Item::new("state");
