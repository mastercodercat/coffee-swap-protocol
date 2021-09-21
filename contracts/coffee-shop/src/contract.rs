use std::ops::{Add, Mul};

use cosmwasm_std::{
    Addr, attr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Uint128, WasmMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{TOKEN_INFO, TokenInfo};
use cw2::set_contract_version;

use crate::coffee_state::{COFFEE_STATE, CoffeeState};
use crate::error::ContractError;
use crate::error::ContractError::InvalidParam;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::products;
use crate::products::{
    AVERAGE_CUP_WEIGHT, Coffee, CoffeeCup, CoffeeRecipe, Ingredient, IngredientPortion,
    WEIGHT_PRECISION,
};
use crate::products::{IngredientCupShare, IngredientsResponse, MenuResponse, OwnerResponse};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:coffee-shop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TOKEN_NAME: &str = "astroport";
const TOKEN_SYMBOL: &str = "ASTRO";

const COFFEE_SHOP_KEY: &str = "coffee-shop";
const DEFAULT_PRICE: Uint128 = Uint128::new(100000000);
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
    msg.validate()?;

    // Option 1
    // store token info using cw20-base format
    // let token_info = TokenInfo {
    //     name: msg.name,
    //     symbol: msg.symbol,
    //     decimals: msg.decimals,
    //     total_supply: Uint128::zero(),
    //     // set self as minter, so we can properly execute mint and burn
    //     mint: Some(MinterData {
    //         minter: _env.contract.address,
    //         cap: None,
    //     }),
    // };
    // TOKEN_INFO.save(deps.storage, &token_info)?;

    // Option 2
    // let token_init_msg = cw20_base::msg::InstantiateMsg {
    //     name: msg.name,
    //     symbol: msg.symbol,
    //     decimals: msg.decimals,
    //     initial_balances: vec![],
    //     // set self as minter, so we can properly execute mint and burn
    //     mint: Some(MinterResponse {
    //         minter: _env.contract.address.to_string(),
    //         cap: None,
    //     }),
    //     marketing: None
    // };
    // cw20_base::contract::instantiate(deps, _env.clone(), info, token_init_msg);

    let state = State {
        owner: info.sender.clone(),
        balance: Uint128::zero(),
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
                recipe: CoffeeRecipe {
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
            },
            CoffeeCup {
                name: String::from(LATE),
                price: DEFAULT_PRICE,
                recipe: CoffeeRecipe {
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
            },
            CoffeeCup {
                name: String::from(AMERICANO),
                price: DEFAULT_PRICE,
                recipe: CoffeeRecipe {
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
            },
        ],
    };

    COFFEE_STATE.save(deps.storage, String::from(COFFEE_SHOP_KEY), &coffee_state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // custom queries
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::Menu { coffee_shop_key } => to_binary(&query_menu(deps, coffee_shop_key)?),
        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
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
        ExecuteMsg::SetPrice { coffee_shop_key, id, price } => set_price(deps, info, coffee_shop_key, id, price),
        ExecuteMsg::BuyCoffee { coffee_shop_key, id, amount } => buy_coffee(deps, info, coffee_shop_key, id, amount),
    }
}

pub fn buy_coffee(
    deps: DepsMut,
    info: MessageInfo,
    coffee_shop_key: String,
    id: Uint128,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let coffee_state = COFFEE_STATE.load(deps.storage, coffee_shop_key.clone());

    if !coffee_state.is_ok() {
        return Err(ContractError::InvalidParam {});
    }
    let val = coffee_state.unwrap();

    let _id = id.u128() as usize;
    if _id == 0 || _id > val.menu.len() {
        return Err(ContractError::InvalidParam {});
    }

    let cup_price = val.menu[_id - 1].price;

    // todo: check balance
    let query_msg = Cw20QueryMsg::Balance { address: info.sender.to_string() };
    // if balance <= cup_price * amount {
    //     return Err(ContractError::NotEnoughFundsError {});
    // }

    COFFEE_STATE.update(deps.storage, coffee_shop_key, |state| -> Result<_, ContractError> {

        // TODO: check wether menu, ingredients have already been init
        let mut val = state.unwrap();
        let total_ingredients_weight = amount.mul(Uint128::new(AVERAGE_CUP_WEIGHT));

        let err = products::check_weight(
            &val.menu[_id - 1].recipe.ingredients,
            &val.ingredient_portions,
            total_ingredients_weight,
        );

        // todo: decrease ingredients amount, increase contract balance, transfer/burn amount sender's address

        Ok(val)
    })?;

    Ok(Response::new().add_attribute("method", "buy_coffee"))
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

    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key.clone());
    if !state.is_ok() {
        return Err(ContractError::InvalidParam {});
    }
    COFFEE_STATE.update(deps.storage, coffee_shop_key, |_state| -> Result<_, ContractError> {
        // TODO: check wether menu have already been init
        let mut val = _state.unwrap();

        let _id = id.u128() as usize;

        if _id == 0 || _id > val.menu.len() || price == Uint128::zero() {
            return Err(ContractError::InvalidParam {});
        }
        val.menu[id.u128() as usize - 1].price = price;
        Ok(val)
    })?;

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

    COFFEE_STATE.update(deps.storage, coffee_shop_key, |state| -> Result<_, ContractError> {
        let mut val = state.unwrap();
        // TODO: eliminate loading ing-s duplicates. Refactor with map probably
        for portion in portions {
            if portion.weight == Uint128::zero() {
                return Err(ContractError::InvalidParam {});
            }
            for state_portion in val.ingredient_portions.iter_mut() {
                if portion.ingredient == state_portion.ingredient {
                    state_portion.weight.add(portion.weight);
                }
            }
        }
        Ok(val)
    })?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}

fn get_owner(deps: Deps) -> Addr {
    query_owner(deps).unwrap().owner
}

fn get_balance<U: Into<String>>(deps: Deps, addr: U) -> Uint128 {
    query_balance(deps, addr.into()).unwrap().balance
}

fn query_ingredients(deps: Deps, coffee_shop_key: String) -> StdResult<IngredientsResponse> {
    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key)?;
    Ok(IngredientsResponse {
        ingredients: state.ingredient_portions,
    })
}

fn get_ingredients(deps: Deps, coffee_shop_key: String) -> Vec<IngredientPortion> {
    query_ingredients(deps, coffee_shop_key).unwrap().ingredients
}

fn query_menu(deps: Deps, coffee_shop_key: String) -> StdResult<MenuResponse> {
    let state = COFFEE_STATE.load(deps.storage, coffee_shop_key)?;
    Ok(MenuResponse { menu: state.menu })
}

fn get_menu(deps: Deps, coffee_shop_key: String) -> Vec<CoffeeCup> {
    query_menu(deps, coffee_shop_key).unwrap().menu
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies(&[]);
        let creator = String::from("creator");
        let msg = InstantiateMsg {
            name: "DRV Token".to_string(),
            symbol: "DRV".to_string(),
            decimals: 0,
        };
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        // no balance
        assert_eq!(get_balance(deps.as_ref(), &creator), Uint128::zero());
        // owner
        assert_eq!(get_owner(deps.as_ref()), Addr::unchecked(creator));
        // menu
        // assert_eq!(
        //     get_menu(deps.as_ref()),
        //     vec![CoffeeCup {}]
        // );
    }

    #[test]
    fn set_price_test() {
        let mut deps = mock_dependencies(&[]);
        let creator = String::from("creator");

        let msg = InstantiateMsg {
            name: "DRV Token".to_string(),
            symbol: "DRV".to_string(),
            decimals: 0,
        };
        let info = mock_info(&creator, &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let shop_key = String::from(COFFEE_SHOP_KEY);
        let zero_value = Uint128::zero();
        let id = Uint128::from(1u8);
        let msg_zeros = ExecuteMsg::SetPrice {
            coffee_shop_key: shop_key,
            id: zero_value,
            price: zero_value,
        };
        let msg = ExecuteMsg::SetPrice { coffee_shop_key: String::from(COFFEE_SHOP_KEY), id, price: id };
        let info = mock_info(&creator, &[]);

        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        let res = get_menu(deps.as_ref(), String::from(COFFEE_SHOP_KEY));

        assert_eq!(res[id.u128() as usize - 1].price, id);

        // other cases
        let info = mock_info(&creator, &[]);

        execute(deps.as_mut(), mock_env(), info, msg_zeros.clone())
            .expect_err("Must return InvalidParam error");
        // match err {
        //     Err(ContractError::InvalidParam {}) => {}
        //     _ => panic!("Must return InvalidParam error"),
        // }
        // assert_eq!(
        //     err,
        //     ContractError::InvalidParam {});

        let res = get_menu(deps.as_ref(), String::from(COFFEE_SHOP_KEY));
        assert_ne!(res[zero_value.u128() as usize].price, zero_value);
    }
}
