use ethers_core::types::{BlockId, Transaction};
use ethers_providers::{Middleware, Provider, Http};
use revm::{
    db::{CacheDB, EthersDB, StateBuilder},
    inspector_handle_register,
    inspectors::TracerEip3155,
    precompile::Bytes,
    primitives::{AccountInfo, Address, TransactTo, B256, U256, TxKind, TxEnv},
    DatabaseCommit,
    Evm
};
use std::{borrow::BorrowMut, sync::Arc};

pub async fn pre_execute_transaction(client: Arc<Provider<Http>>, tx: &TxEnv) {

    // Params
    let chain_id: u64 = 1;
    let block_number = 10889447;

    // Fetch the transaction-rich block
    let block = client.get_block_with_txs(block_number).await.unwrap().unwrap();
    println!("Fetched block number: {}", block.number.unwrap().0[0]);
    let previous_block_number = block_number - 1;

    // Use the previous block state as the db with caching
    let prev_id: BlockId = previous_block_number.into();
    // SAFETY: This cannot fail since this is in the top-level tokio runtime

    // Using empty because we don't need this info for now
    let tracer = TracerEip3155::new(Box::new(std::io::empty()));

    let state_db = EthersDB::new(client, Some(prev_id)).expect("panic");
    let cache_db: CacheDB<EthersDB<Provider<Http>>> = CacheDB::new(state_db);
    let mut state = StateBuilder::new_with_database(cache_db).build();
    let mut evm = Evm::builder()
        .with_db(&mut state)
        .with_external_context(tracer)
        .modify_block_env(|b| {
            b.number = U256::from(block.number.unwrap().0[0]);
            b.coinbase = Address::from(block.author.unwrap().as_fixed_bytes());
            b.timestamp = U256::from(block.timestamp.0[0]);

            b.difficulty = U256::from(block.difficulty.0[0]);
            b.gas_limit = U256::from(block.gas_limit.0[0]);
        })
        .modify_cfg_env(|c| {
            c.chain_id = chain_id;
        })
        .append_handler_register(inspector_handle_register)
        .build();

    let txs = block.transactions.len();
    println!("Found {txs} transactions.");

    evm = evm
        .modify()
        .modify_tx_env(|etx| {
            etx.clone_from(tx);
        })
        .build();
    
    let evm_result = evm.transact();

    if let Ok(evm_result) = evm_result {
        let revm::primitives::ResultAndState { result, state } = evm_result;

        // what need to be inspected
        dbg!(result);
        dbg!(state);
    } else {
        println!("Something went wrong here. The transaction is invalid");
    }

}

#[cfg(test)]
mod tests {
    use super::pre_execute_transaction;
    use revm::primitives::{Address, TxEnv, Bytes, U256};
    use tokio::runtime::Runtime;
    use ethers_providers::{Provider, Http};
    use std::sync::Arc;
    use ethers_core::types::{Transaction, H256, H160, U64, OtherFields};
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

        rt.block_on( async { pre_execute_transaction(client, &tx).await });
    }
}