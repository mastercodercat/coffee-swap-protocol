use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::products;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub balance: Uint128,
    pub menu: Vec<products::CoffeeCup>,
    pub ingredient_portions: Vec<products::IngredientPortion>,
}

pub const STATE: Item<State> = Item::new("state");
