use cosmwasm_std::{Addr, Uint128, QuerierWrapper, QueryRequest, WasmQuery, CosmosMsg, WasmMsg, StdResult, to_binary, Response};
use cw20::{Cw20QueryMsg, Cw20ExecuteMsg, BalanceResponse};
use crate::error::ContractError;

pub fn query_token_balance(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    let res: BalanceResponse = querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: String::from(account_addr),
            })?,
        }))
        .unwrap_or_else(|_| BalanceResponse {
            balance: Uint128::zero(),
        });

    Ok(res.balance)
}

pub fn execute_transfer(contract_addr: Addr, recipient: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let res = Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount,
            })?,
            funds: vec![],
        }));
    Ok(res)
}

pub fn execute_transfer_from(contract_addr: Addr, owner: Addr, recipient: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let res = Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: owner.to_string(),
                recipient: recipient.to_string(),
                amount,
            })?,
            funds: vec![],
        }));
    Ok(res)
}