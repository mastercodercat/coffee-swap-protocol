use cw_storage_plus::Map;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

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
pub enum Ingredient {
    Sugar,
    Milk,
    Water,

    // Coffee beans
    Arabica,
    Robusta,
    Liberica,
    Excelsa,
}

// impl PartialEq for Ingredient {
//     fn eq(&self, other: &Ingredient) -> bool {
//         self == other
//     }
// }

// impl CoffeeRecipe {
// fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     write!(f, "{} of {}", self.relative_volume, self.name)
// }
// let mut recipe: String;
// for (name, relative_volume) in self.ingredients {
// let str: String = format!("{}", relative_volume);
// recipe.insert_str(recipe.len()-1, format!("{} of {}", relative_volume, name))
// }
// return recipe;
//     }
// }
