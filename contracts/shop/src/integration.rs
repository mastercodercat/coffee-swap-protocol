#[cfg(test)]
mod tests {
    use std::ops::{Add, Mul, Sub};

    use anyhow::{anyhow, Result as AnyHowResult};
    use cosmwasm_std::{Addr, attr, ContractResult, Empty, QueryRequest, Response, StdError, to_binary, Uint128, WasmMsg, WasmQuery};
    use cosmwasm_vm::testing::execute as vm_testing_execute;
    use cw20::{BalanceResponse, MinterResponse};
    use cw20_base::msg::{ExecuteMsg as Cw20ExecuteMsg, QueryMsg as Cw20QueryMsg};
    // use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
    use cw20_base::state::{MinterData, TOKEN_INFO, TokenInfo};
    use cw_multi_test::{App, AppResponse, BankKeeper, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::products::{AVERAGE_CUP_WEIGHT, calculate_total_ingredient_weight, Ingredient, IngredientPortion, IngredientsResponse, MenuResponse,
                          PriceResponse, RecipesResponse, SHARE_PRECISION};

    const ALICE: &str = "Alice";
    const BOB: &str = "Bob";

    fn mock_app() -> App {
        App::default()
    }

    pub fn contract_coffee_swap() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    pub fn contract_cw20_token() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    fn mint_some_token(
        router: &mut App,
        owner: Addr,
        token_instance: Addr,
        to: &str,
        amount: Uint128,
    ) {
        let recipient = String::from(to);

        // mint some cw20 tokens for buying
        let cw20_mint_msg = Cw20ExecuteMsg::Mint {
            recipient: recipient.clone(),
            amount,
        };

        let res = router
            .execute_contract(owner.clone(), token_instance.clone(), &cw20_mint_msg, &[])
            .unwrap();
        assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
        assert_eq!(res.events[1].attributes[2], attr("to", recipient.clone()));
        assert_eq!(res.events[1].attributes[3], attr("amount", amount));
    }

    fn allowance_token(router: &mut App, owner: Addr, spender: Addr, token: Addr, amount: Uint128) {
        let msg = cw20::Cw20ExecuteMsg::IncreaseAllowance {
            spender: spender.to_string(),
            amount,
            expires: None,
        };
        let res = router
            .execute_contract(owner.clone(), token.clone(), &msg, &[])
            .unwrap();
        assert_eq!(
            res.events[1].attributes[1],
            attr("action", "increase_allowance")
        );
        assert_eq!(
            res.events[1].attributes[2],
            attr("owner", owner.to_string())
        );
        assert_eq!(
            res.events[1].attributes[3],
            attr("spender", spender.to_string())
        );
        assert_eq!(res.events[1].attributes[4], attr("amount", amount));
    }

    fn check_balance(router: &mut App, user: Addr, token: Addr, expected_amount: Uint128) {
        let msg = Cw20QueryMsg::Balance {
            address: user.to_string(),
        };

        let res: Result<BalanceResponse, _> =
            router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: token.to_string(),
                msg: to_binary(&msg).unwrap(),
            }));

        let balance = res.unwrap();

        assert_eq!(balance.balance, expected_amount);
    }

    fn set_price_test(router: &mut App, sender: Addr, contract: Addr, shop_key: String, id: Uint128, price: Uint128) {
        let set_price_msg = ExecuteMsg::SetPrice {
            coffee_shop_key: shop_key.clone(),
            id,
            price,
        };

        router
            .execute_contract(
                sender.clone(),
                contract.clone(),
                &set_price_msg,
                &[],
            );

        // compare set price
        let price_query = QueryMsg::Price {
            coffee_shop_key: shop_key.clone(),
            id,
        };
        let res: PriceResponse = router
            .wrap()
            .query_wasm_smart(&contract.clone(), &price_query)
            .unwrap();

        assert_eq!(res.price, price);
    }

    #[test]
    fn should_allow_buy_if_coffee_shop_init() {
        let mut router = mock_app();

        let owner = Addr::unchecked("owner");

        // setup cw20 token, coffee-swap
        let cw20_token_id = router.store_code(contract_cw20_token());

        let cw20_instantiate_msg = cw20_base::msg::InstantiateMsg {
            name: "Token".parse().unwrap(),
            symbol: "TKN".parse().unwrap(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: owner.to_string(),
                cap: None,
            }),
            marketing: None,
        };
        let cw20_addr = router
            .instantiate_contract(
                cw20_token_id,
                owner.clone(),
                &cw20_instantiate_msg,
                &[],
                "Token",
                None,
            )
            .unwrap();

        let alice_address = Addr::unchecked(ALICE);
        let amount = Uint128::from(u128::pow(10, 6));

        // mint tokens for Alice
        mint_some_token(&mut router, owner.clone(), cw20_addr.clone(), ALICE, amount);
        check_balance(&mut router, alice_address.clone(), cw20_addr.clone(), amount);

        let price = Uint128::new(91);
        let allowed_spend_amount = amount.mul(amount);
        let coffee_swap_id = router.store_code(contract_coffee_swap());
        let shop_key = "astro".to_string();
        let coffee_cup_id = Uint128::new(1);

        let msg = InstantiateMsg {
            token_addr: cw20_addr.clone(),
            shop_key: shop_key.clone(),
        };
        let coffee_swap_addr = router
            .instantiate_contract(coffee_swap_id, owner.clone(), &msg, &[], "Token", None)
            .unwrap();

        set_price_test(&mut router, owner.clone(), coffee_swap_addr.clone(), shop_key.clone(), coffee_cup_id, price);

        let portions = vec![
            IngredientPortion {
                ingredient: Ingredient::Beans,
                weight: Uint128::new(1000),
            },
            IngredientPortion {
                ingredient: Ingredient::Water,
                weight: Uint128::new(1000),
            },
            IngredientPortion {
                ingredient: Ingredient::Milk,
                weight: Uint128::new(1000),
            },
            IngredientPortion {
                ingredient: Ingredient::Sugar,
                weight: Uint128::new(1000),
            },
        ];

        let load_msg = ExecuteMsg::LoadIngredients {
            coffee_shop_key: shop_key.clone(),
            portions: portions.clone(),
        };

        // user can't load ingredients
        let res = router
            .execute_contract(
                alice_address.clone(),
                coffee_swap_addr.clone(),
                &load_msg,
                &[],
            )
            .unwrap_err();
        assert_eq!(res.to_string(), "Unauthorized");

        // owner loads ingredients
        router.execute_contract(owner.clone(), coffee_swap_addr.clone(), &load_msg, &[]);

        // check the load was successful
        let ingredients_query = QueryMsg::Ingredients {
            coffee_shop_key: shop_key.clone(),
        };
        let ingredients: IngredientsResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr, &ingredients_query)
            .unwrap();

        assert_eq!(ingredients.ingredients, portions.clone());

        // router
        //     .execute_contract(bob_address.clone(), coffee_swap_addr.clone(), &buy_msg, &[])
        //     .expect_err("Must return NotEnoughFunds error");

        // save balances before buy/sell
        // let cw20_buyer_balance_query = Cw20QueryMsg::Balance {
        //     address: alice_address.to_string(),
        // };
        // let cw20_contract_balance_query = Cw20QueryMsg::Balance {
        //     address: coffee_swap_addr.to_string(),
        // };
        // let buyer_balance_before: BalanceResponse = router
        //     .wrap()
        //     .query_wasm_smart(&cw20_addr, &cw20_buyer_balance_query)
        //     .unwrap();
        // let balance_before: BalanceResponse = router
        //     .wrap()
        //     .query_wasm_smart(&cw20_addr, &cw20_contract_balance_query)
        //     .unwrap();

        let bob_address = Addr::unchecked(BOB);
        let cup_amount = Uint128::new(2);
        let infinite_amount = Uint128::from(u128::pow(10, 22));

        let buy_msg = ExecuteMsg::BuyCoffee {
            coffee_shop_key: shop_key.clone(),
            id: coffee_cup_id.clone(),
            amount: infinite_amount,
        };

        let res = router
            .execute_contract(bob_address.clone(), coffee_swap_addr.clone(), &buy_msg, &[])
            .unwrap_err();
        assert_eq!(res.to_string(), "NotEnoughIngredients");

        let buy_msg = ExecuteMsg::BuyCoffee {
            coffee_shop_key: shop_key.clone(),
            id: coffee_cup_id.clone(),
            amount: cup_amount.clone(),
        };
        allowance_token(&mut router, alice_address.clone(), coffee_swap_addr.clone(), cw20_addr.clone(), allowed_spend_amount);

        // user can't buy with NotEnoughFunds
        // let res = router
        //     .execute_contract(bob_address.clone(), coffee_swap_addr.clone(), &buy_msg, &[])
        //     .unwrap_err();
        // assert_eq!(res.to_string(), "NotEnoughFunds");


        // allowance_token(&mut app, bob_address.clone(), coffee_swap_addr.clone(),cw20_addr.clone(), allowed_spend_amount);

        // user without set allowance can't buy
        let res = router
            .execute_contract(bob_address.clone(), coffee_swap_addr.clone(), &buy_msg, &[])
            .unwrap_err();
            assert_eq!(res.to_string(), "NotEnoughFunds");

        // user buys coffee successfully
        let res = router
            .execute_contract(
                alice_address.clone(),
                coffee_swap_addr.clone(),
                &buy_msg,
                &[],
            )
            .unwrap_err();
        // assert_eq!(res.to_string(), "123");

        // check decreasing ingredients portions
        let ingredients_query = QueryMsg::Recipes {
            coffee_shop_key: shop_key.clone(),
        };
        let recipes: RecipesResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr, &ingredients_query)
            .unwrap();
        let ingredients_for_selected_cup = recipes.recipes[coffee_cup_id.u128() as usize - 1]
            .ingredients
            .clone();

        // ingredients_before_sell
        let total_ingredients_weight = cup_amount.mul(Uint128::new(AVERAGE_CUP_WEIGHT));
        let mut ingredients_remained = ingredients.ingredients;
        for ingredient in ingredients_remained.iter_mut() {
            for ingredient_portion in ingredients_for_selected_cup.clone() {
                if ingredient.ingredient != ingredient_portion.ingredient_type {
                    continue;
                }
                ingredient.weight = ingredient
                    .weight
                    .checked_sub(calculate_total_ingredient_weight(
                        total_ingredients_weight,
                        ingredient_portion.share,
                        SHARE_PRECISION,
                    ))
                    .unwrap();
            }
        }
        let ingredients_query = QueryMsg::Ingredients {
            coffee_shop_key: shop_key.clone(),
        };
        let ingredients_after_sell: IngredientsResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr, &ingredients_query)
            .unwrap();

        assert_eq!(ingredients_after_sell.ingredients, ingredients_remained);
        assert_eq!(ingredients_after_sell.ingredients, &[]);
        assert_eq!(ingredients_remained, &[]);

        // assert_eq!(ingredients_before_sell.ingredients, ingredients_after_sell.ingredients );
    }
}
