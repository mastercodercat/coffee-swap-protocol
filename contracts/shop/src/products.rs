use std::ops::{Add, Div, Mul};

use cosmwasm_std::{Addr, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// share like %
pub const SHARE_PRECISION: Uint128 = Uint128::new(100);

pub const AVERAGE_CUP_WEIGHT: u128 = 250u128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoffeeCup {
    pub name: String,
    pub price: Uint128,
}

pub fn check_weight(
    ingredients: &Vec<IngredientCupShare>,
    portions: &Vec<IngredientPortion>,
    weight: Uint128,
    share_precision: Uint128,
) -> bool {
    for ingredient in ingredients.iter() {
        for portion in portions.iter() {
            if ingredient.ingredient_type != portion.ingredient {
                continue;
            }
            if portion.weight
                < calculate_total_ingredient_weight(weight, ingredient.share, share_precision)
            {
                return false;
            }
        }
    }
    return true;
}

pub fn calculate_total_ingredient_weight(
    total_ingredients_weight: Uint128,
    ingredient_share: Uint128,
    share_precision: Uint128,
) -> Uint128 {
    return total_ingredients_weight
        .mul(ingredient_share)
        .checked_div(share_precision)
        .unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoffeeRecipe {
    // simplified example: Late { Water: 0.5, Milk: 0.3, Beans: 0.15, Sugar: 0.05 }
    // todo: make water and beans required components, add relative_volume
    pub ingredients: Vec<IngredientCupShare>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Coffee {
    Cappuccino,
    Late,
    Americano,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IngredientCupShare {
    pub ingredient_type: Ingredient,
    // percentages (100% is max)
    pub share: Uint128,
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
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PriceResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MenuResponse {
    pub menu: Vec<CoffeeCup>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RecipesResponse {
    pub recipes: Vec<CoffeeRecipe>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IngredientsResponse {
    pub ingredients: Vec<IngredientPortion>,
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint128;

    use crate::products::{check_weight, Ingredient, IngredientCupShare, IngredientPortion, SHARE_PRECISION, calculate_total_ingredient_weight};

    #[test]
    fn check_calculate() {
        let weight = Uint128::new(100);
        let share = Uint128::new(45);
        let total = calculate_total_ingredient_weight(weight, share, SHARE_PRECISION);

        assert_eq!(total, share)
    }

    #[test]
    fn check_weight_test() {
        let ingredient_portions = vec![
            IngredientPortion {
                ingredient: Ingredient::Beans,
                weight: Uint128::new(100),
            },
            IngredientPortion {
                ingredient: Ingredient::Water,
                weight: Uint128::new(100),
            },
            IngredientPortion {
                ingredient: Ingredient::Milk,
                weight: Uint128::new(100),
            },
            IngredientPortion {
                ingredient: Ingredient::Sugar,
                weight: Uint128::new(100),
            },
        ];
        let ingredients = vec![
            IngredientCupShare {
                ingredient_type: Ingredient::Water,
                share: Uint128::new(45),
            },
            IngredientCupShare {
                ingredient_type: Ingredient::Beans,
                share: Uint128::new(25),
            },
            IngredientCupShare {
                ingredient_type: Ingredient::Milk,
                share: Uint128::new(25),
            },
            IngredientCupShare {
                ingredient_type: Ingredient::Sugar,
                share: Uint128::new(5),
            },
        ];

        assert_eq!(
            check_weight(
                &ingredients,
                &ingredient_portions,
                Uint128::new(100),
                SHARE_PRECISION
            ),
            true
        );

        assert_eq!(
            check_weight(
                &ingredients,
                &ingredient_portions,
                Uint128::new(200),
                SHARE_PRECISION
            ),
            true
        );

        assert_eq!(
            check_weight(
                &ingredients,
                &ingredient_portions,
                Uint128::new(1000),
                SHARE_PRECISION
            ),
            false
        );
    }
}
