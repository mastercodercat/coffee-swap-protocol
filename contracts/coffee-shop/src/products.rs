use cosmwasm_std::Uint128;
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoffeeCup {
    pub name: String,
    pub recipe: CoffeeRecipe,
    pub price: Uint128,
    // volume: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoffeeRecipe {
    // simplified example: Late { Water: 0.5, Milk: 0.3, Beans: 0.15, Sugar: 0.05 }
    // todo: make water and beans required components, add relative_volume
    pub ingredients: Vec<Ingredient>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Coffee {
    Cappuccino,
    Late,
    Americano,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IngredientPortion {
    pub ingredient: Ingredient,
    pub weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Ingredient {
    Sugar,
    Milk,
    Water,
    Beans,
    // Coffee beans
    // Arabica,
    // Robusta,
    // Liberica,
    // Excelsa,
}
