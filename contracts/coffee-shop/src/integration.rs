use astroport::staking::{ConfigResponse, InstantiateMsg as xInstatiateMsg, QueryMsg};
use astroport::token::InstantiateMsg;
use cosmwasm_std::{
    attr,
    testing::{mock_env, MockApi, MockStorage},
    to_binary, Addr, QueryRequest, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw_multi_test::{App, BankKeeper, ContractWrapper, Executor};

const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const CAROL: &str = "Carol";

fn mock_app() -> App {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();

    App::new(api, env.block, bank, MockStorage::new())
}

fn instantiate_contracts(router: &mut App, owner: Addr) -> (Addr, Addr, Addr) {
    let astro_token_contract = Box::new(ContractWrapper::new(
        astroport_token::contract::execute,
        astroport_token::contract::instantiate,
        astroport_token::contract::query,
    ));

    let astro_token_code_id = router.store_code(astro_token_contract);

    let msg = InstantiateMsg {
        name: String::from("Astro token"),
        symbol: String::from("ASTRO"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        init_hook: None,
    };

    let astro_token_instance = router
        .instantiate_contract(
            astro_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ASTRO"),
            None,
        )
        .unwrap();

    let staking_contract = Box::new(ContractWrapper::new(
        astroport_staking::contract::execute,
        astroport_staking::contract::instantiate,
        astroport_staking::contract::query,
    ));
    let staking_code_id = router.store_code(staking_contract);

    let msg = xInstatiateMsg {
        token_code_id: astro_token_code_id,
        deposit_token_addr: astro_token_instance.clone(),
    };
    let staking_instance = router
        .instantiate_contract(
            staking_code_id,
            owner,
            &msg,
            &[],
            String::from("xASTRO"),
            None,
        )
        .unwrap();

    let msg = QueryMsg::Config {};
    let x_astro_token_instance = router
        .wrap()
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: staking_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }))
        .unwrap()
        .share_token_addr;

    (
        astro_token_instance,
        staking_instance,
        x_astro_token_instance,
    )
}

fn mint_some_astro(router: &mut App, owner: Addr, astro_token_instance: Addr, to: &str) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: String::from(to),
        amount: Uint128::from(100u128),
    };
    let res = router
        .execute_contract(owner.clone(), astro_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", String::from(to)));
    assert_eq!(
        res.events[1].attributes[3],
        attr("amount", Uint128::from(100u128))
    );
}

#[test]
fn should_not_allow_byu_if_not_enough_tokens() {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");

    let (astro_token_instance, staking_instance, x_astro_token_instance) =
        instantiate_contracts(&mut router, owner.clone());

    // mint 100 ASTRO for Alice
    mint_some_astro(
        &mut router,
        owner.clone(),
        astro_token_instance.clone(),
        ALICE,
    );
    let alice_address = Addr::unchecked(ALICE);

    // check if Alice's ASTRO balance is 100
    let msg = Cw20QueryMsg::Balance {
        address: alice_address.to_string(),
    };
    let res: Result<BalanceResponse, _> =
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: astro_token_instance.to_string(),
            msg: to_binary(&msg).unwrap(),
        }));
    assert_eq!(
        res.unwrap(),
        BalanceResponse {
            balance: Uint128::from(100u128)
        }
    );

}