use axum::{
    extract::State,
    routing::post,
    Router,
    Json
};
use std::sync::Arc;
use ethers_providers::{Provider, Http};
use serde::Deserialize;
use revm::primitives::{Address, TxEnv, Bytes, U256};
use std::str::FromStr;

pub mod configs;
pub mod simulator;

#[tokio::main]
async fn main() {
    // TODO: initialize logs


    // get configuration from config.toml
    let config = configs::read_config();

    // setup our RPC client
    let client = Provider::<Http>::try_from(config.ethereumrpc).unwrap();
    let client = Arc::new(client);

    // build our application with a route
    // TODO: add our endpoint to receive transactions
    let app = Router::new()
        .route("/execute", post(handle_execute))
        .with_state(client);

    // run our app with hyper, listening globally on port 3000
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("port to be free");
    axum::serve(listener, app).await.expect("server to start");
}

// pass a transaction in the EVM to simulate its execution
async fn handle_execute(
    State(client): State<Arc<Provider<Http>>>,
    Json(tx): Json<TransactionPayload>,
) {
    let transaction = TxEnv {
        nonce: Some(tx.nonce),
        caller: Address::from_str(&tx.from).expect("proper address"), // TODO: improve error handling
        transact_to: revm::primitives::TxKind::Call(Address::from_str(&tx.to).expect("proper address")), // TODO: improve error handling
        value: U256::from_str_radix(&tx.value, 10).expect("expect correct value"), // TODO: improve error handling
        gas_price: U256::from_str_radix(&tx.gas_price, 10).expect("expect valid gas_price"), // TODO: improve error handling
        gas_limit: tx.gas_limit,
        data: Bytes::from_str(&tx.data).expect("expect hex data correctly formated"),
        access_list: Default::default(),
        gas_priority_fee: None,
        chain_id: Some(1), // we are only doing mainnet
        max_fee_per_blob_gas: None,
        blob_hashes: Default::default(),
    };
    let _ = simulator::execute_transaction(client, &transaction);
}

#[derive(Deserialize)]
struct TransactionPayload { 
    from: String,
    to: String,
    value: String,
    nonce: u64,
    gas_price: String,
    gas_limit: u64,
    data: String,
    gas_priority_fee: String,
    max_fee_per_blob_gas: String

}