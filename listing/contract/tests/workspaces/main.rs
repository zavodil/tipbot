use serde_json::json;
use near_units::{parse_gas};
use workspaces::{Worker};
use workspaces::network::Sandbox;

use workspaces::prelude::*;

const CONTRACT_WASM_FILEPATH: &str = "";//./../out/local.wasm";

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
        }))?
        .gas(parse_gas!("100 T") as u64)
        .transact()
        .await?;

    println!("init contract outcome: {:#?}", _outcome_init);

    Ok(())
}