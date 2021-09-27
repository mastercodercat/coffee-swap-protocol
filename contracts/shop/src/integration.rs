#[cfg(test)]
mod tests {
    use cosmwasm_std::{attr, to_binary, Addr, Empty, QueryRequest, Uint128, WasmMsg, WasmQuery, ContractResult, Response};
    use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
    use cw20_base::msg;
    use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
    use cw_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
    use cosmwasm_vm::testing::{
        execute as vm_testing_execute,
        // instantiate, mock_backend_with_balances, mock_env, query, MockApi, MockQuerier,
        // MockStorage, MOCK_CONTRACT_ADDR,
    };

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::products::{Ingredient, IngredientPortion, IngredientsResponse, PriceResponse};
    use crate::ContractError;
    use std::ops::{Mul, Add};

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

    fn mint_some_astro(
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

    #[test]
    fn should_allow_buy_if_coffee_shop_inited() {
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
        let amount = Uint128::from(1000u128);
        let allowed_spend_amount = amount.mul(amount);

        // mint tokens for Alice
        mint_some_astro(&mut router, owner.clone(), cw20_addr.clone(), ALICE, amount);

        let price = Uint128::from(100u128);
        // let res: Result<BalanceResponse, _> =
        //     router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
        //         contract_addr: cw20_addr.to_string(),
        //         msg: to_binary(&cw20_balance_query).unwrap(),
        //     }));
        // assert_eq!(
        //     res.unwrap(),
        //     BalanceResponse {
        //         balance: amount
        //     }
        // );

        // check if Alice's ASTRO balance is 100
        let cw20_balance_query = Cw20QueryMsg::Balance {
            address: alice_address.to_string(),
        };
        let balance: BalanceResponse = router
            .wrap()
            .query_wasm_smart(&cw20_addr, &cw20_balance_query)
            .unwrap();

        // compare minted amount
        assert_eq!(balance.balance, amount);

        let coffee_swap_id = router.store_code(contract_coffee_swap());

        let shop_key = "astro".to_string();

        let _msg = InstantiateMsg {
            token_addr: cw20_addr.clone(),
            shop_key: shop_key.clone(),
        };
        let coffee_swap_addr = router
            .instantiate_contract(coffee_swap_id, owner.clone(), &_msg, &[], "Token", None)
            .unwrap();

        let set_price_msg = ExecuteMsg::SetPrice {
            coffee_shop_key: shop_key.clone(),
            id: Uint128::new(1u128),
            price,
        };

        // user can't set price
        let res =
            router.execute_contract(
                alice_address.clone(),
                coffee_swap_addr.clone(),
                &set_price_msg,
                &[],
            ).unwrap_err();
        // assert_eq!(res, ContractError::Unauthorized {});

        // owner sets price
        router.execute_contract(owner.clone(), coffee_swap_addr.clone(), &set_price_msg, &[]);

        // compare set price
        let coffee_cup_id = Uint128::new(1u128);
        let price_query = QueryMsg::Price {
            coffee_shop_key: shop_key.clone(),
            id: coffee_cup_id,
        };
        let res: PriceResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr.clone(), &price_query)
            .unwrap();

        assert_eq!(res.price, price);

        let portions = vec![
            IngredientPortion {
                ingredient: Ingredient::Beans,
                weight: Uint128::new(1000u128),
            },
            IngredientPortion {
                ingredient: Ingredient::Water,
                weight: Uint128::new(1000u128),
            },
            IngredientPortion {
                ingredient: Ingredient::Milk,
                weight: Uint128::new(1000u128),
            },
            IngredientPortion {
                ingredient: Ingredient::Sugar,
                weight: Uint128::new(1000u128),
            },
        ];

        let load_msg = ExecuteMsg::LoadIngredients {
            coffee_shop_key: shop_key.clone(),
            portions: portions.clone(),
        };

        // user loads ingredients
        // router
        //     .execute_contract(
        //         alice_address.clone(),
        //         coffee_swap_addr.clone(),
        //         &load_msg,
        //         &[],
        //     )
        //     .expect_err("Must return Unauthorised error");

        // let res: ContractResult<Response> = vm_testing_execute(deps, env, info, load_msg);
        // assert_eq!(res.unwrap_err(), "Unauthorized");

        // owner loads ingredients
        router.execute_contract(owner.clone(), coffee_swap_addr.clone(), &load_msg, &[]);

        // check the load was successful
        let ingredients_query = QueryMsg::Ingredients {
            coffee_shop_key: shop_key.clone(),
        };
        let ingredients_before_sell: IngredientsResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr, &ingredients_query)
            .unwrap();

        assert_eq!(ingredients_before_sell.ingredients, portions.clone());

        // user without set allowance can't buy

        // user with zero balance can't buy
        // router
        //     .execute_contract(bob_address.clone(), coffee_swap_addr.clone(), &buy_msg, &[])
        //     .expect_err("Must return NotEnoughFunds error");

        // save balances before buy/sell
        let cw20_buyer_balance_query = Cw20QueryMsg::Balance {
            address: alice_address.to_string(),
        };
        let cw20_contract_balance_query = Cw20QueryMsg::Balance {
            address: coffee_swap_addr.to_string(),
        };
        let buyer_balance_before: BalanceResponse = router
            .wrap()
            .query_wasm_smart(&cw20_addr, &cw20_buyer_balance_query)
            .unwrap();
        let balance_before: BalanceResponse = router
            .wrap()
            .query_wasm_smart(&cw20_addr, &cw20_contract_balance_query)
            .unwrap();

        let bob_address = Addr::unchecked(BOB);
        let cup_amount = Uint128::new(2);
        let buy_msg = ExecuteMsg::BuyCoffee {
            coffee_shop_key: shop_key.clone(),
            id: coffee_cup_id.clone(),
            amount: cup_amount.clone(),
        };

        let balance: BalanceResponse = router
            .wrap()
            .query_wasm_smart(&cw20_addr.clone(), &cw20_balance_query)
            .unwrap();

        let set_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
            spender: cw20_addr.to_string(),
            amount: allowed_spend_amount,
            expires: None
        };
        router.execute_contract(alice_address.clone(), cw20_addr.clone(), &set_allowance_msg, &[]);

        // user buys coffee successfully
        router.execute_contract(
            alice_address.clone(),
            coffee_swap_addr.clone(),
            &buy_msg,
            &[],
        );

        // check balances, ingredient portions,
        let cw20_buyer_balance_query = Cw20QueryMsg::Balance {
            address: alice_address.to_string(),
        };
        let cw20_contract_balance_query = Cw20QueryMsg::Balance {
            address: coffee_swap_addr.to_string(),
        };
        let buyer_balance_after: BalanceResponse = router
            .wrap()
            .query_wasm_smart(&cw20_addr.clone(), &cw20_buyer_balance_query)
            .unwrap();
        let balance_after: BalanceResponse = router
            .wrap()
            .query_wasm_smart(&cw20_addr.clone(), &cw20_contract_balance_query)
            .unwrap();

        // compare amounts
        let total = price.mul(cup_amount);
        assert_eq!(balance_after.balance, total.add(balance_before.balance));
        assert_eq!(buyer_balance_after.balance, total.add(buyer_balance_before.balance));

        assert_eq!(ingredients_before_sell.ingredients, )// ingredients_after_sell.ingredients );
    }
}
