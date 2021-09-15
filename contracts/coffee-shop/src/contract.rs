#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
// use cosmwasm_std::{
//     coin, to_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, DistributionMsg, Env,
//     MessageInfo, QuerierWrapper, Response, StakingMsg, StdError, StdResult, Uint128, WasmMsg,
// };
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{MenuResponse, ExecuteMsg, InstantiateMsg, OwnerResponse, QueryMsg};
use crate::products::{CoffeeRecipe, Ingredient, Coffee, CoffeeCup};
use crate::state::{State, STATE};

use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

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
    // store token info using cw20-base format
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: _env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    let state = State {
        owner: info.sender.clone(),
        balance: Uint128::zero(),
        minted_amount: Uint128::zero(),
        // recipes: vec![CoffeeRecipe {
        //     ingredients: vec![
        //         Ingredient::Arabica,
        //         Ingredient::Water,
        //         Ingredient::Milk,
        //         Ingredient::Sugar,
        //     ],
        // }],
        menu: vec![CoffeeCup {
            name: String::from(CAPPUCCINO),
            price: DEFAULT_PRICE,
            recipe: CoffeeRecipe {
                    ingredients: vec![
                        Ingredient::Arabica,
                        Ingredient::Water,
                        Ingredient::Milk,
                        Ingredient::Sugar,
                    ],
            }
        }],
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

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

#[cfg(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) {
    match msg {
        ExecuteMsg::SetPrice {id, price} => set_price(deps, id, price),
    }
}


#[cfg(not(feature = "library"), entry_point)]
pub fn set_price(deps: Deps, id: Uint128, price: Uint128) -> Result<Response, ContractError> {
    
}

//
// pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
//     STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//         state.count += 1;
//         Ok(state)
//     })?;
//
//     Ok(Response::new().add_attribute("method", "try_increment"))
// }
// pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
//     STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//         if info.sender != state.owner {
//             return Err(ContractError::Unauthorized {});
//         }
//         state.count = count;
//         Ok(state)
//     })?;
//     Ok(Response::new().add_attribute("method", "reset"))
// }
//
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
//     }
// }

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}

fn get_owner(deps: Deps) -> Addr {
    query_owner(deps).unwrap().owner
}

// fn query_recipes(deps: Deps) -> StdResult<CoffeeListResponse> {
//     let state = STATE.load(deps.storage)?;
//     Ok(CoffeeListResponse {
//         list: state.recipes,
//     })
// }

// fn get_recipes(deps: Deps) -> Vec<CoffeeRecipe> {
//     query_recipes(deps).unwrap().list
// }

fn get_balance<U: Into<String>>(deps: Deps, addr: U) -> Uint128 {
    query_balance(deps, addr.into()).unwrap().balance
}

fn query_menu(deps: Deps) -> StdResult<MenuResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(MenuResponse {
        menu: state.menu,
    })
}

fn get_menu(deps: Deps) -> Vec<CoffeeCup> {
    query_menu(deps).unwrap().menu
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

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

    }
}

//
//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(&coins(2, "token"));
//
//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//
//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//
//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }
//
//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(&coins(2, "token"));
//
//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//
//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }
//
//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();
//
//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
