use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    address_hex, async_call_tx_input, async_callback_tx_input, merge_results,
    try_execute_builtin_function,
    tx_mock::{TxCache, TxContext, TxContextRef, TxInput, TxOutput, TxResult},
    world_mock::{AccountData, AccountEsdt, BlockchainMock, BlockchainMockError},
    AsyncCallTxData, ContractMap, DebugApi,
};

use super::execute_tx_context;

pub fn sc_query(
    tx_input: TxInput,
    state: Rc<BlockchainMock>,
    contract_map: &ContractMap<DebugApi>,
) -> TxResult {
    let tx_context = TxContextRef::new(tx_input, state);
    execute_tx_context(tx_context, contract_map)
}

pub fn sc_call(
    tx_input: TxInput,
    state: &mut Rc<BlockchainMock>,
    contract_map: &ContractMap<DebugApi>,
    increase_nonce: bool, // TODO: flag = code smell, refactor!
) -> Result<TxResult, BlockchainMockError> {
    if let Some(tx_result) = try_execute_builtin_function(&tx_input, state) {
        return Ok(tx_result);
    }

    let func_name_empty = tx_input.func_name.is_empty();
    let tx_context = TxContextRef::new(tx_input, state.clone());

    if increase_nonce {
        tx_context
            .blockchain_cache
            .increase_acount_nonce(&tx_context.tx_input_box.from);
    }
    tx_context.blockchain_cache.subtract_egld_balance(
        &tx_context.tx_input_box.from,
        &tx_context.tx_input_box.egld_value,
    )?;
    tx_context.blockchain_cache.subtract_tx_gas(
        &tx_context.tx_input_box.from,
        tx_context.tx_input_box.gas_limit,
        tx_context.tx_input_box.gas_price,
    );
    tx_context.blockchain_cache.increase_egld_balance(
        &tx_context.tx_input_box.to,
        &tx_context.tx_input_box.egld_value,
    );

    let tx_result = if func_name_empty {
        // direct EGLD transfer
        TxResult::empty()
    } else {
        execute_tx_context(tx_context.clone(), contract_map)
    };

    let blockchain_updates = tx_context.into_blockchain_updates();

    if tx_result.result_status == 0 {
        blockchain_updates.apply(Rc::get_mut(state).unwrap());
    }

    // if tx_result.result_status == 0 {
    //     // replace storage with new one
    //     let _ = std::mem::replace(&mut contract_account.storage, tx_output.contract_storage);

    //     state.increase_egld_balance(&to, &egld_value);
    //     state.increase_multi_esdt_balance(&to, esdt_values.as_slice());

    //     state.send_balance(
    //         &to,
    //         tx_output.send_balance_list.as_slice(),
    //         &mut tx_result.result_logs,
    //     )?;
    // } else {
    //     // revert
    //     state.increase_egld_balance(&from, &egld_value);
    //     state.increase_multi_esdt_balance(&from, esdt_values.as_slice());
    // }

    // Ok((tx_result, tx_output.async_call))
    Ok(tx_result)
}

pub fn sc_call_with_async_and_callback(
    tx_input: TxInput,
    state: &mut Rc<BlockchainMock>,
    contract_map: &ContractMap<DebugApi>,
    increase_nonce: bool,
) -> Result<TxResult, BlockchainMockError> {
    let contract_address = tx_input.to.clone();
    let mut tx_result = sc_call(tx_input, state, contract_map, increase_nonce)?;
    if tx_result.result_status == 0 {
        if let Some(async_data) = tx_result.async_call.clone() {
            if state.accounts.contains_key(&async_data.to) {
                let async_input = async_call_tx_input(&async_data, &contract_address);

                let async_result =
                    sc_call_with_async_and_callback(async_input, state, contract_map, false)?;

                tx_result = merge_results(tx_result, async_result.clone());

                let callback_input =
                    async_callback_tx_input(&async_data, &contract_address, &async_result);
                let callback_result = sc_call(callback_input, state, contract_map, false)?;
                assert!(
                    callback_result.async_call.is_none(),
                    "successive asyncs currently not supported"
                );
                tx_result = merge_results(tx_result, callback_result);
            } else {
                // panic!("async cross shard simuation not supported")
                let tx_cache = TxCache::new(state.clone());
                tx_cache
                    .subtract_egld_balance(&contract_address, &async_data.call_value)
                    .unwrap();
                tx_cache.insert_account(AccountData {
                    address: async_data.to.clone(),
                    nonce: 0,
                    egld_balance: async_data.call_value,
                    esdt: AccountEsdt::default(),
                    username: Vec::new(),
                    storage: HashMap::new(),
                    contract_path: None,
                    contract_owner: None,
                });
                state.commit_tx_cache(tx_cache);
            }
        }
    }
    Ok(tx_result)
}
