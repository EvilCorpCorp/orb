use std::error::Error;
use ethers_providers::{Provider, Http};
use revm::{
    db::{CacheDB, EthersDB, StateBuilder},
    inspector_handle_register,
    inspectors::TracerEip3155,
    primitives::{U256, TxEnv, ResultAndState},
    Evm
};
use std::{sync::Arc};


const CHAIN_ID: u64 = 1;

pub async fn execute_transaction(client: Arc<Provider<Http>>, block_number: u64, tx: &TxEnv) -> Result<ResultAndState, Box<dyn Error>> {
    // Using empty because we don't need this info for now
    let tracer = TracerEip3155::new(Box::new(std::io::empty()));

    let state_db = EthersDB::new(client, Some(block_number.into())).expect("panic");
    let cache_db: CacheDB<EthersDB<Provider<Http>>> = CacheDB::new(state_db);
    let mut state = StateBuilder::new_with_database(cache_db).build();
    let mut evm = Evm::builder()
        .with_db(&mut state)
        .with_external_context(tracer)
        .modify_block_env(|b| {
            b.number = U256::from(block_number + 1);
        })
        .modify_cfg_env(|c| {
            c.chain_id = CHAIN_ID;
        })
        .append_handler_register(inspector_handle_register)
        .build();

    evm = evm
        .modify()
        .modify_tx_env(|etx| {
            etx.clone_from(tx);
        })
        .build();
    
    evm.transact()
        .map_err(|_err| {
            "Something went wrong here. The transaction is invalid.".into()
        })
}

#[cfg(test)]
mod tests {
    use super::execute_transaction;
    use revm::primitives::{Address, TxEnv, Bytes, U256};
    use tokio::runtime::Runtime;
    use ethers_providers::{Provider, Http};
    use std::sync::Arc;
    use std::str::FromStr;

    #[test]
    fn test_simulator() {
        let rt  = Runtime::new().unwrap();

        // Set up the HTTP transport which is consumed by the RPC client.
        let client = Provider::<Http>::try_from("http://95.217.34.145:8545").unwrap();
        let client = Arc::new(client);

        let tx = TxEnv {
            nonce: Some(6676),
            caller: Address::from_str("0x4f69c5b694d5a14a0a595703175c478ec6b2a2fe").unwrap(),
            transact_to: revm::primitives::TxKind::Call(Address::from_str("0xa0b5d75fef7b024294411cd92bf7e68ba5f18c99").unwrap()),
            value: U256::from(0),
            gas_price: U256::from_str_radix("440741668508", 10).unwrap(),
            gas_limit: 450000,
            data: Bytes::from_str("0x94264cbc00000000000000000000000000000000000000000000000070035f89301680000000000000000000000000000000000000000000000000004050cfbc87c12f29000000000000000000000000000000000000000000000000000000005f655db2000000000000000000000000000000000000000000000000000000005f6558a4").unwrap(),
            access_list: Default::default(),
            gas_priority_fee: None,
            chain_id: Some(1),
            max_fee_per_blob_gas: None,
            blob_hashes: Default::default(),
        };

        let block_number = 10889447 - 1;

        rt.block_on( async { execute_transaction(client, block_number, &tx).await });
    }
}