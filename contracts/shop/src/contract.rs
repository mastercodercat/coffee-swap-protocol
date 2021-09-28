use std::ops::{Add, Mul, Sub};

use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary,
    Uint128,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::allowances::execute_transfer_from as cw20_transfer_from;
use cw20_base::contract::{
    execute as cw20_execute, execute_burn, execute_mint, execute_send,
    execute_transfer as cw20_execute_transfer, query as cw20_query,
    query_balance as cw20_query_balance,
};
use cw20_base::state::{MinterData, TOKEN_INFO, TokenInfo};
use cw2::set_contract_version;

use crate::coffee_state::{COFFEE_STATE, CoffeeState};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::msg::QueryMsg::Price;
use crate::products;
use crate::products::{AVERAGE_CUP_WEIGHT, calculate_total_ingredient_weight, Coffee, CoffeeCup, CoffeeRecipe, Ingredient, IngredientPortion, PriceResponse, RecipesResponse, SHARE_PRECISION};
use crate::products::{IngredientCupShare, IngredientsResponse, MenuResponse, OwnerResponse};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:shop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_PRICE: Uint128 = Uint128::new(1000);

// coffee menu
const CAPPUCCINO: &str = "Cappuccino";
const LATE: &str = "Late";
const AMERICANO: &str = "Americano";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        balance: Uint128::zero(),
        coffee_token_addr: deps.api.addr_validate(&msg.token_addr.to_string())?,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    let coffee_state = CoffeeState {
        ingredient_portions: vec![
            IngredientPortion {
                ingredient: Ingredient::Beans,
                weight: Uint128::zero(),
            },
            IngredientPortion {
                ingredient: Ingredient::Water,
                weight: Uint128::zero(),
            },
            IngredientPortion {
                ingredient: Ingredient::Milk,
                weight: Uint128::zero(),
            },
            IngredientPortion {
                ingredient: Ingredient::Sugar,
                weight: Uint128::zero(),
            },
        ],
        menu: vec![
            CoffeeCup {
                name: String::from(CAPPUCCINO),
                price: DEFAULT_PRICE,
            },
            CoffeeCup {
                name: String::from(LATE),
                price: DEFAULT_PRICE,
            },
            CoffeeCup {
                name: String::from(AMERICANO),
                price: DEFAULT_PRICE,
            },
        ],
        recipes: vec![
            CoffeeRecipe {
                ingredients: vec![
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
                ],
            },
            CoffeeRecipe {
                ingredients: vec![
                    IngredientCupShare {
                        ingredient_type: Ingredient::Beans,
                        share: Uint128::new(2),
                    },
                    IngredientCupShare {
                        ingredient_type: Ingredient::Water,
                        share: Uint128::new(45),
                    },
                    IngredientCupShare {
                        ingredient_type: Ingredient::Beans,
                        share: Uint128::new(25),
                    },
                ],
            },
            CoffeeRecipe {
                ingredients: vec![
                    IngredientCupShare {
                        ingredient_type: Ingredient::Water,
                        share: Uint128::new(70),
                    },
                    IngredientCupShare {
                        ingredient_type: Ingredient::Beans,
                        share: Uint128::new(25),
                    },
                    IngredientCupShare {
                        ingredient_type: Ingredient::Sugar,
                        share: Uint128::new(5),
                    },
                ],
            },
        ],
    };

    COFFEE_STATE.save(deps.storage, String::from(msg.shop_key), &coffee_state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.clone()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // custom queries
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::Balance {} => to_binary(&query_balance(deps)?),
        QueryMsg::Price {
            coffee_shop_key,
            id,
        } => to_binary(&query_price(deps, coffee_shop_key, id)?),
        QueryMsg::Menu { coffee_shop_key } => to_binary(&query_menu(deps, coffee_shop_key)?),
        QueryMsg::Recipes { coffee_shop_key } => to_binary(&query_recipes(deps, coffee_shop_key)?),
        QueryMsg::Ingredients { coffee_shop_key } => {
            to_binary(&query_ingredients(deps, coffee_shop_key)?)
        }
        // inherited from cw20-base
        QueryMsg::CW20Balance { address } => to_binary(&cw20_query_balance(deps, address)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetPrice {
            coffee_shop_key,
            id,
            price,
        } => set_price(deps, info, coffee_shop_key, id, price),
        ExecuteMsg::LoadIngredients {
            coffee_shop_key,
            portions,
        } => load_ingredients(deps, info, coffee_shop_key, portions),
        ExecuteMsg::BuyCoffee {
            coffee_shop_key,
            id,
            amount,
        } => buy_coffee(deps, info, _env, coffee_shop_key, id, amount),
    }
}

pub fn buy_coffee(
    mut deps: DepsMut,
    info: MessageInfo,
    env: Env,
    coffee_shop_key: String,
    id: Uint128,
    cup_amount: Uint128,
) -> Result<Response, ContractError> {
    let coffee_state = COFFEE_STATE.load(deps.storage, coffee_shop_key.clone())?;

    let _id = id.u128() as usize;
    if _id == 0 || _id > coffee_state.menu.len() {
        return Err(ContractError::InvalidParam {});
    }

    let coffee_state = COFFEE_STATE.load(deps.storage, coffee_shop_key.clone())?;

    let cup_price = coffee_state.menu[_id - 1].price;

    // check is enough ingredients for order
    let recipe = coffee_state.recipes[_id - 1].clone();
    let total_ingredients_weight = cup_amount.mul(Uint128::new(AVERAGE_CUP_WEIGHT));

    let is_enough_ingredients = products::check_weight(
        &recipe.ingredients,
        &coffee_state.ingredient_portions,
        total_ingredients_weight,
        SHARE_PRECISION,
    );
    if !is_enough_ingredients {
        return Err(ContractError::NotEnoughIngredients {});
    }
    // transfer amount from sender to contract balance
    let total = cup_amount.mul(cup_price);

    cw20_transfer_from(
        deps.branch(),
        env.clone(),
        info.clone(),
        info.sender.to_string(),
        env.contract.address.to_string(),
        total,
    )?;

    // decrease ingredients amount
    COFFEE_STATE.update(
        deps.storage,
        coffee_shop_key.clone(),
        |state| -> Result<_, ContractError> {
            let mut val = state.unwrap();
            for portion in val.ingredient_portions.iter_mut() {
                for ingredient in recipe.ingredients.iter() {
                    if ingredient.ingredient_type != portion.ingredient {
                        continue;
                    }
                    portion.weight = portion.weight.checked_sub(calculate_total_ingredient_weight(
                        total_ingredients_weight,
                        ingredient.share,
                        SHARE_PRECISION,
                    )).unwrap();
                }
            }
            Ok(val)
        },
    )?;

    Ok(Response::new().add_attribute("method", "buy_coffee"))
}

fn query_balance(deps: Deps) -> StdResult<BalanceResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(BalanceResponse {
        balance: state.balance,
    })
}

pub fn set_price(
    deps: DepsMut,
    info: MessageInfo,
    coffee_shop_key: String,
    id: Uint128,
    price: Uint128,
) -> Result<Response, ContractError> {
    if info.sender != STATE.load(deps.storage)?.owner {
        return Err(ContractError::Unauthorized {});
    }

    COFFEE_STATE.update(
        deps.storage,
        coffee_shop_key,
        |state| -> Result<_, ContractError> {
            // TODO: check wether menu have already been init
            let mut val = state.unwrap();

            let _id = id.u128() as usize;

            if _id == 0 || _id > val.menu.len() || price == Uint128::zero() {
                return Err(ContractError::InvalidParam {});
            }
            val.menu[_id - 1].price = price;
            Ok(val)
        },
    )?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

pub fn load_ingredients(
    deps: DepsMut,
    info: MessageInfo,
    coffee_shop_key: String,
    portions: Vec<IngredientPortion>,
) -> Result<Response, ContractError> {
    if info.sender != STATE.load(deps.storage)?.owner {
        return Err(ContractError::Unauthorized {});
    }

    COFFEE_STATE.update(
        deps.storage,
        coffee_shop_key,
        |state| -> Result<_, ContractError> {
            let mut val = state.unwrap();
            // TODO: eliminate loading ing-s duplicates. Refactor with map
            for portion in portions {
                if portion.weight == Uint128::zero() {
                    return Err(ContractError::InvalidParam {});
                }
                for state_portion in val.ingredient_portions.iter_mut() {
                    if portion.ingredient == state_portion.ingredient {
                        state_portion.weight = state_portion.weight.add(portion.weight);
                    }
                }
            }
            Ok(val)
        },
    )?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}

fn query_ingredients(deps: Deps, coffee_shop_key: String) -> StdResult<IngredientsResponse> {
    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key)?;
    Ok(IngredientsResponse {
        ingredients: state.ingredient_portions,
    })
}

fn query_price(deps: Deps, coffee_shop_key: String, id: Uint128) -> StdResult<PriceResponse> {
    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key)?;
    let _id = id.u128() as usize;
    if _id == 0 || _id > state.menu.len() {
        // return Err(NotFound {});
    }
    Ok(PriceResponse {
        price: state.menu[_id - 1].price,
    })
}

fn query_menu(deps: Deps, coffee_shop_key: String) -> StdResult<MenuResponse> {
    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key)?;
    Ok(MenuResponse { menu: state.menu })
}

fn query_recipes(deps: Deps, coffee_shop_key: String) -> StdResult<RecipesResponse> {
    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key)?;
    Ok(RecipesResponse {
        recipes: state.recipes,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies(&[]);
        let creator = String::from("creator");
        let shop_key = "shop".to_string();

        let msg = InstantiateMsg {
            token_addr: Addr::unchecked("addr"),
            shop_key,
        };
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        // owner
        assert_eq!(
            query_owner(deps.as_ref()).unwrap().owner,
            Addr::unchecked(creator)
        );
    }

    #[test]
    fn set_price_test() {
        let mut deps = mock_dependencies(&[]);
        let creator = String::from("creator");
        let shop_key = "shop".to_string();

        let msg = InstantiateMsg {
            token_addr: Addr::unchecked("addr"),
            shop_key: shop_key.clone(),
        };
        let info = mock_info(&creator, &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let zero_value = Uint128::zero();
        let id = Uint128::new(1);
        let msg_zeros = ExecuteMsg::SetPrice {
            coffee_shop_key: shop_key.clone(),
            id: zero_value,
            price: zero_value,
        };
        let msg = ExecuteMsg::SetPrice {
            coffee_shop_key: shop_key.clone(),
            id,
            price: id,
        };

        execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());

        let menu = query_menu(deps.as_ref(), shop_key.clone()).unwrap().menu;

        assert_ne!(menu[zero_value.u128() as usize].price, zero_value);
        assert_eq!(menu[id.u128() as usize - 1].price, id);

        let res = execute(deps.as_mut(), mock_env(), info, msg_zeros.clone()).unwrap_err();
        assert_eq!(res.to_string(), "InvalidParam");
    }
}
