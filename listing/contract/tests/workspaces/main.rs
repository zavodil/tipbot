use serde_json::json;
use near_units::{parse_gas};
use workspaces::{Worker};
use workspaces::network::Sandbox;
use near_sdk::{AccountId};

use workspaces::prelude::*;
use near_sdk::json_types::U128;

const CONTRACT_WASM_FILEPATH: &str = "./../listing/out/local.wasm";
const TIPTOKEN_ACCOUNT_ID: &str = "tiptoken.near";
const LISTING_PAYMENT: u128 = 1_000_000_000_000_000_000_000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker: Worker<Sandbox> = workspaces::sandbox().await?;
    let owner = worker.root_account();

    let contract_wasm = std::fs::read(CONTRACT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&contract_wasm).await?;

    let _outcome_init = contract
        .call(&worker, "new")
        .args_json(json!({
                "owner_id": owner.id(),
                "tiptoken_account_id": AccountId::new_unchecked(TIPTOKEN_ACCOUNT_ID.to_string()),
                "listing_payment": U128::from(LISTING_PAYMENT)
        }))?
        .gas(parse_gas!("100 T") as u64)
        .transact()
        .await?;

    println!("init contract outcome: {:#?}", _outcome_init);

    Ok(())
}