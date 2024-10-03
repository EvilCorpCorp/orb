use ethers_core::types::BlockId;
use ethers_providers::{Middleware, Provider, Http};
use revm::{
    db::{CacheDB, EthersDB, StateBuilder},
    inspector_handle_register,
    inspectors::TracerEip3155,
    precompile::Bytes,
    primitives::{AccountInfo, Address, TransactTo, B256, U256, TxKind},
    DatabaseCommit,
    Evm
};
use std::sync::Arc;

pub async fn pre_execute_transaction() {

    // Set up the HTTP transport which is consumed by the RPC client.
    let client = Provider::<Http>::try_from("http://95.217.34.145:8545").unwrap();
    let client = Arc::new(client);

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

    for tx in block.transactions {
        evm = evm
            .modify()
            .modify_tx_env(|etx| {
                etx.caller = Address::from(tx.from.as_fixed_bytes());
                etx.gas_limit = tx.gas.as_u64();
                etx.gas_price = U256::from_limbs(
                    tx.gas_price.unwrap().0
                );
                etx.value = U256::from_limbs(tx.value.0);
                etx.data = tx.input.0.into();
                etx.gas_priority_fee = if let Some(max_priority_fee_per_gas) = tx.max_priority_fee_per_gas { Some(U256::from_limbs(max_priority_fee_per_gas.0)) } else { None };
                etx.chain_id = Some(chain_id);
                etx.nonce = Some(tx.nonce.as_u64());
                etx.access_list = Default::default();

                etx.transact_to = match tx.to {
                    Some(to_address) => TxKind::Call(Address::from(to_address.as_fixed_bytes())),
                    None => TxKind::Create,
                };
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
}

#[cfg(test)]
mod tests {
    use super::pre_execute_transaction;
    use tokio::runtime::Runtime;

    #[test]
    fn test_simulator() {
        let rt  = Runtime::new().unwrap();

        rt.block_on( async { pre_execute_transaction().await });
    }
}