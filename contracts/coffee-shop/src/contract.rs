use std::ops::{Add, Mul};
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

use crate::error::ContractError;
use crate::error::ContractError::InvalidParam;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::products;
use crate::products::{
    Coffee, CoffeeCup, CoffeeRecipe, Ingredient, IngredientPortion, AVERAGE_CUP_WEIGHT,
    WEIGHT_PRECISION,
};
use crate::products::{IngredientCupShare, IngredientsResponse, MenuResponse, OwnerResponse};

use crate::state::{State, STATE};
use crate::coffee_state::{CoffeeState, COFFEE_STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:coffee-shop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_PRICE: Uint128 = Uint128::new(1);
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
    // Check valid token info
    msg.validate()?;

    // store token info using cw20-base format
    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: _env.contract.address,
            cap: None,
        }),
        // mint: match msg.mint {
        //     Some(m) => Some(MinterData {
        //         minter: deps.api.addr_validate(&m.minter)?,
        //         cap: m.cap,
        // }),
        // None => None,
        // };
    };
    TOKEN_INFO.save(deps.storage, &token_info)?;

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

    COFFEE_STATE.save(deps.storage, &coffee_state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // custom queries
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::Menu {} => to_binary(&query_menu(deps)?),
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
        ExecuteMsg::SetPrice { id, price } => set_price(deps, info, id, price),
        ExecuteMsg::BuyCoffee { id, amount } => buy_coffee(deps, info, id, amount),
    }
}

pub fn buy_coffee(
    deps: DepsMut,
    info: MessageInfo,
    id: Uint128,
    amount: Uint128,
) -> Result<Response, ContractError> {
    COFFEE_STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        let index = id.u128() as usize;
        // TODO: check wether menu, ingredients have already been init
        if state.menu.len() > index {
            return Err(ContractError::InvalidParam {});
        }

        let order_total = amount.mul(Uint128::new(AVERAGE_CUP_WEIGHT));

        let err = products::check_loaded_ingredients_weight(
            state.menu[index - 1].recipe.ingredients,
            state.ingredient_portions,
            order_total,
        );

        match weight {
            // NotAnError {} => {}
            InvalidParam {} => {
                return Err(InvalidParam);
            }
        };

        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "buy_coffee"))
}

pub fn set_price(
    deps: DepsMut,
    info: MessageInfo,
    id: Uint128,
    price: Uint128,
) -> Result<Response, ContractError> {
    COFFEE_STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        if id == Uint128::zero() || price == Uint128::zero() {
            return Err(ContractError::InvalidParam {});
        }
        // TODO: check wether menu have already been init
        state.menu[id.u128() as usize - 1].price = price;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

pub fn load_ingredients(
    deps: DepsMut,
    info: MessageInfo,
    portions: Vec<IngredientPortion>,
) -> Result<Response, ContractError> {
    COFFEE_STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        // TODO: eliminate loading ing-s duplicates. Refactor with map probably
        for portion in portions {
            if portion.weight == Uint128::zero() {
                return Err(ContractError::InvalidParam {});
            }
            for state_portion in state.ingredient_portions.iter_mut() {
                if portion.ingredient == state_portion.ingredient {
                    state_portion.weight.add(portion.weight);
                }
            }
        }
        Ok(state)
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

fn query_ingredients(deps: Deps) -> StdResult<IngredientsResponse> {
    let state = COFFEE_STATE.load(deps.storage)?;
    Ok(IngredientsResponse {
        ingredients: state.ingredient_portions,
    })
}

fn get_ingredients(deps: Deps) -> Vec<IngredientPortion> {
    query_ingredients(deps).unwrap().ingredients
}

fn query_menu(deps: Deps) -> StdResult<MenuResponse> {
    let state = COFFEE_STATE.load(deps.storage)?;
    Ok(MenuResponse { menu: state.menu })
}

fn get_menu(deps: Deps) -> Vec<CoffeeCup> {
    query_menu(deps).unwrap().menu
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
        let creator = String::from("creeator");

        let msg = InstantiateMsg {
            name: "DRV Token".to_string(),
            symbol: "DRV".to_string(),
            decimals: 0,
        };
        let info = mock_info(&creator, &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let zero_value = Uint128::zero();
        let id = Uint128::from(1u8);
        let msg_zeros = ExecuteMsg::SetPrice {
            id: zero_value,
            price: zero_value,
        };
        let msg = ExecuteMsg::SetPrice { id: id, price: id };
        let info = mock_info(&creator, &[]);

        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        let res = get_menu(deps.as_ref());

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

        let res = get_menu(deps.as_ref());
        assert_ne!(res[zero_value.u128() as usize].price, zero_value);
    }
}
