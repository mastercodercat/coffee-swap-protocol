use crate::products::{CoffeeCup, CoffeeRecipe, IngredientPortion};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoffeeState {
    pub menu: Vec<CoffeeCup>,
    pub recipes: Vec<CoffeeRecipe>,
    pub ingredient_portions: Vec<IngredientPortion>,
}

pub const COFFEE_STATE: Map<String, CoffeeState> = Map::new("coffee_state");
