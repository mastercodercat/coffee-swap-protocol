#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, attr, Empty, QueryRequest, to_binary, Uint128, WasmMsg, WasmQuery};
    use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
    use cw20_base::msg;
    use cw20_base::state::{MinterData, TOKEN_INFO, TokenInfo};
    use cw_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::products::{Ingredient, IngredientPortion, IngredientsResponse, PriceResponse};

    const ALICE: &str = "Alice";

    fn mock_app() -> App {
        App::default()
    }

    pub fn contract_coffee_swap() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            execute, instantiate, query,
        );
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

    fn mint_some_astro(router: &mut App, owner: Addr, token_instance: Addr, to: &str, amount: Uint128) {
        let recipient = String::from(to);

        // mint some cw20 tokens for buying
        let cw20_mint_msg = Cw20ExecuteMsg::Mint {
            recipient: recipient.clone(),
            amount,
        };

        let res = router
            .execute_contract(
                owner.clone(),
                token_instance.clone(),
                &cw20_mint_msg,
                &[],
            )
            .unwrap();
        assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
        assert_eq!(res.events[1].attributes[2], attr("to", recipient.clone()));
        assert_eq!(res.events[1].attributes[3], attr("amount", amount));
    }

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
        let amount = Uint128::from(100u128);
        // mint 100 tokens for Alice
        mint_some_astro(
            &mut router,
            owner.clone(),
            cw20_addr.clone(),
            ALICE,
            amount,
        );

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

        let _msg = InstantiateMsg { token_addr: cw20_addr.clone(), shop_key: shop_key.clone() };
        let coffee_swap_addr = router
            .instantiate_contract(
                coffee_swap_id,
                owner.clone(),
                &_msg,
                &[],
                "Token",
                None,
            ).unwrap();

        let set_price_msg = ExecuteMsg::SetPrice {
            coffee_shop_key: shop_key.clone(),
            id: Uint128::new(1u128),
            price: amount,
        };

        // user sets price
        router
            .execute_contract(alice_address.clone(), coffee_swap_addr.clone(), &set_price_msg, &[])
            .expect_err("Must return Unauthorised error");

        // owner sets price
        router
            .execute_contract(owner.clone(), coffee_swap_addr.clone(), &set_price_msg, &[]);

        // compare set price
        let coffee_cup_id = Uint128::new(1u128);
        let price_query = QueryMsg::Price {
            coffee_shop_key: shop_key.clone(),
            id: coffee_cup_id,
        };
        let price: PriceResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr.clone(), &price_query)
            .unwrap();

        assert_eq!(price.price, amount);

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
        router
            .execute_contract(alice_address.clone(), coffee_swap_addr.clone(), &load_msg, &[])
            .expect_err("Must return Unauthorised error");

        // owner loads ingredients
        router
            .execute_contract(owner.clone(), coffee_swap_addr.clone(), &load_msg, &[]);

        // check the load was successful
        let ingredients_query = QueryMsg::Ingredients {
            coffee_shop_key: shop_key
        };
        let ingredients: IngredientsResponse = router
            .wrap()
            .query_wasm_smart(&coffee_swap_addr, &ingredients_query)
            .unwrap();

        assert_eq!(ingredients.ingredients, portions.clone());


    }
}