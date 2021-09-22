use cosmwasm_std::{
    Addr,
    attr,
    QueryRequest, testing::{mock_env, MockApi, MockStorage}, to_binary, Uint128, WasmQuery,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
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

fn mint_some_astro(router: &mut App, owner: Addr, astro_token_instance: Addr, to: &str) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: String::from(to),
        amount: Uint128::from(100),
    };
    let res = router
        .execute_contract(owner.clone(), astro_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", String::from(to)));
    assert_eq!(
        res.events[1].attributes[3],
        attr("amount", Uint128::from(100))
    );
}

#[test]
fn should_not_allow_buy_if_not_enough_tokens() {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");

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
            balance: Uint128::from(100)
        }
    );
}