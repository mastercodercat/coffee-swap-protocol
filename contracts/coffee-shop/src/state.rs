use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub balance: Uint128,
    pub coffee_token_addr: Addr,
}

pub const STATE: Item<State> = Item::new("state");
