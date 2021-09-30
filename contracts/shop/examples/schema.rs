use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use shop::coffee_state::CoffeeState;
use shop::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use shop::products::{IngredientsResponse, MenuResponse, OwnerResponse};
use shop::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(CoffeeState), &out_dir);
    export_schema(&schema_for!(OwnerResponse), &out_dir);
    export_schema(&schema_for!(MenuResponse), &out_dir);
    export_schema(&schema_for!(IngredientsResponse), &out_dir);
}
