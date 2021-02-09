use super::big_uint_api_mock::*;
use crate::async_data::AsyncCallTxData;
use crate::{SendBalance, TxContext, TxPanic, TxOutput};
use elrond_wasm::api::{ContractHookApi, SendApi};
use elrond_wasm::types::{Address, ArgBuffer, BoxedBytes, CodeMetadata, TokenIdentifier};
use num_bigint::BigUint;
use num_traits::Zero;

impl TxContext {
	fn get_available_egld_balance(&self) -> BigUint {
		// start with the pre-existing balance
		let mut available_balance = self.blockchain_info_box.contract_balance.clone();

		// add amount received
		available_balance += &self.tx_input_box.call_value;

		// already sent
		let tx_output = self.tx_output_cell.borrow();
		for send_balance in &tx_output.send_balance_list {
			available_balance -= &send_balance.amount;
		}

		available_balance
	}

	fn get_available_esdt_balance(&self, token_name: &[u8]) -> BigUint {
		// start with the pre-existing balance
		let mut available_balance = self
			.blockchain_info_box
			.contract_esdt
			.get(token_name)
			.unwrap_or(&BigUint::zero())
			.clone();

		// add amount received (if the same token)
		if self.tx_input_box.esdt_token_name == token_name {
			available_balance += &self.tx_input_box.esdt_value;
		}
		let tx_output = self.tx_output_cell.borrow();

		// already sent
		for send_balance in &tx_output.send_balance_list {
			if send_balance.token == token_name {
				available_balance -= &send_balance.amount;
			}
		}

		available_balance
	}
}

impl SendApi<RustBigUint> for TxContext {
	fn direct_egld(&self, to: &Address, amount: &RustBigUint, _data: &[u8]) {
		if &amount.value() > &self.get_available_egld_balance() {
			panic!(TxPanic {
				status: 10,
				message: b"failed transfer (insufficient funds)".to_vec(),
			});
		}

		let mut tx_output = self.tx_output_cell.borrow_mut();
		tx_output.send_balance_list.push(SendBalance {
			recipient: to.clone(),
			token: TokenIdentifier::egld(),
			amount: amount.value(),
		})
	}

	fn direct_esdt_execute(
		&self,
		to: &Address,
		token: &[u8],
		amount: &RustBigUint,
		_gas: u64,
		_function: &[u8],
		_arg_buffer: &ArgBuffer,
	) {
		if &amount.value() > &self.get_available_esdt_balance(token) {
			panic!(TxPanic {
				status: 10,
				message: b"insufficient funds".to_vec(),
			});
		}

		let mut tx_output = self.tx_output_cell.borrow_mut();
		tx_output.send_balance_list.push(SendBalance {
			recipient: to.clone(),
			token: TokenIdentifier::from(token),
			amount: amount.value(),
		})
	}

	fn async_call_raw(&self, to: &Address, amount: &RustBigUint, data: &[u8]) -> ! {
		// the cell is no longer needed, since we end in a panic
		let mut tx_output = self.tx_output_cell.replace(TxOutput::default());
		tx_output.async_call = Some(AsyncCallTxData {
			to: to.clone(),
			call_value: amount.value(),
			call_data: data.to_vec(),
			tx_hash: self.get_tx_hash(),
		});
		panic!(tx_output)
	}

	fn deploy_contract(
		&self,
		_gas: u64,
		_amount: &RustBigUint,
		_code: &BoxedBytes,
		_code_metadata: CodeMetadata,
		_arg_buffer: &ArgBuffer,
	) -> Address {
		panic!("deploy_contract not yet implemented")
	}

	fn execute_on_dest_context(
		&self,
		_gas: u64,
		_address: &Address,
		_value: &RustBigUint,
		_function: &[u8],
		_arg_buffer: &ArgBuffer,
	) {
		panic!("execute_on_dest_context not implemented yet!");
	}

	fn execute_on_dest_context_by_caller(
		&self,
		_gas: u64,
		_address: &Address,
		_value: &RustBigUint,
		_function: &[u8],
		_arg_buffer: &ArgBuffer,
	) {
		panic!("execute_on_dest_context_by_caller not implemented yet!");
	}

	fn execute_on_same_context(
		&self,
		_gas: u64,
		_address: &Address,
		_value: &RustBigUint,
		_function: &[u8],
		_arg_buffer: &ArgBuffer,
	) {
		panic!("execute_on_same_context not implemented yet!");
	}
}
