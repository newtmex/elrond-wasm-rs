use elrond_wasm::{Address, ArgBuffer, Box, BoxedBytes, CodeMetadata, Vec, H256};

use crate::big_int::*;
use crate::big_uint::*;
// use crate::ext_error;
use elrond_wasm::api::*;
use elrond_wasm::err_msg;
use elrond_wasm::BigUintApi;
use elrond_wasm::ContractHookApi;

const ADDRESS_LENGTH: usize = 32;
const TOPIC_LENGTH: usize = 32;

#[rustfmt::skip]
extern {
    fn getSCAddress(resultOffset: *mut u8);
    fn getOwnerAddress(resultOffset: *mut u8);
    fn blockHash(nonce: i64, resultOffset: *mut u8) -> i32;
    fn getNumArguments() -> i32;
    fn getArgumentLength(id: i32) -> i32;
    fn getArgument(id: i32, dstOffset: *mut u8) -> i32;
    fn getFunction(functionOffset: *const u8) -> i32;
    fn storageStore(keyOffset: *const u8, keyLength: i32, dataOffset: *const u8, dataLength: i32) -> i32;
    fn storageLoadLength(keyOffset: *const u8, keyLength: i32) -> i32;
    fn storageLoad(keyOffset: *const u8, keyLength: i32, dataOffset: *mut u8) -> i32;

    fn transferValue(dstOffset: *const u8, valueOffset: *const u8, dataOffset: *const u8, length: i32) -> i32;
	fn asyncCall(dstOffset: *const u8, valueOffset: *const u8, dataOffset: *const u8, length: i32);
	fn createContract(gas: u64,
		valueOffset: *const u8,
		codeOffset: *const u8, codeMetadataOffset: *const u8, length: i32,
		resultOffset: *const u8,
		numArguments: i32, argumentsLengthOffset: *const u8, dataOffset: *const u8) -> i32;

    fn getCaller(resultOffset: *mut u8);
    fn callValue(resultOffset: *const u8) -> i32;
    fn getESDTValue(resultOffset: *const u8) -> usize;
    fn getESDTTokenName(resultOffset: *const u8) -> usize;
    fn writeLog(pointer: *const u8, length: i32, topicPtr: *const u8, numTopics: i32);
    fn finish(dataOffset: *const u8, length: i32);

    fn getGasLeft() -> i64;
    fn getBlockTimestamp() -> i64;
    fn getBlockNonce() -> i64;
    fn getBlockRound() -> i64;
    fn getBlockEpoch() -> i64;
    fn getBlockRandomSeed(resultOffset: *mut u8);
    fn getStateRootHash(resultOffset: *mut u8);
    fn getPrevBlockTimestamp() -> i64;
    fn getPrevBlockNonce() -> i64;
    fn getPrevBlockRound() -> i64;
    fn getPrevBlockEpoch() -> i64;
    fn getPrevBlockRandomSeed(resultOffset: *const u8);
    fn getOriginalTxHash(resultOffset: *const u8);

    // big int API
    fn bigIntNew(value: i64) -> i32;
    fn bigIntStorageStoreUnsigned(keyOffset: *const u8, keyLength: i32, source: i32) -> i32;
    fn bigIntStorageLoadUnsigned(keyOffset: *const u8, keyLength: i32, destination: i32) -> i32;
    fn bigIntGetExternalBalance(address_ptr: *const u8, dest: i32);
    fn bigIntGetUnsignedArgument(arg_id: i32, dest: i32);
    fn bigIntGetSignedArgument(arg_id: i32, dest: i32);
    fn bigIntGetCallValue(dest: i32);
    fn bigIntGetESDTCallValue(dest: i32);
    fn bigIntFinishUnsigned(bih: i32);
    fn bigIntFinishSigned(bih: i32);

    // small int API
    fn smallIntGetUnsignedArgument(id: i32) -> i64;
    fn smallIntGetSignedArgument(id: i32) -> i64;
    fn smallIntFinishUnsigned(value: i64);
    fn smallIntFinishSigned(value: i64);
    fn smallIntStorageStoreUnsigned(keyOffset: *const u8, keyLength: i32, value: i64) -> i32;
    fn smallIntStorageStoreSigned(keyOffset: *const u8, keyLength: i32, value: i64) -> i32;
    fn smallIntStorageLoadUnsigned(keyOffset: *const u8, keyLength: i32) -> i64;
    fn smallIntStorageLoadSigned(keyOffset: *const u8, keyLength: i32) -> i64;
    
    // crypto API
    fn sha256(dataOffset: *const u8, length: i32, resultOffset: *mut u8) -> i32;
    fn keccak256(dataOffset: *const u8, length: i32, resultOffset: *mut u8) -> i32;
}

pub struct ArwenApiImpl {}
impl elrond_wasm::ContractHookApi<ArwenBigInt, ArwenBigUint> for ArwenApiImpl {
	#[inline]
	fn get_sc_address(&self) -> Address {
		unsafe {
			let mut res = Address::zero();
			getSCAddress(res.as_mut_ptr());
			res
		}
	}

	#[inline]
	fn get_owner_address(&self) -> Address {
		unsafe {
			let mut res = Address::zero();
			getOwnerAddress(res.as_mut_ptr());
			res
		}
	}

	#[inline]
	fn get_caller(&self) -> Address {
		unsafe {
			let mut res = Address::zero();
			getCaller(res.as_mut_ptr());
			res
		}
	}

	fn get_balance(&self, address: &Address) -> ArwenBigUint {
		unsafe {
			let result = bigIntNew(0);
			bigIntGetExternalBalance(address.as_ref().as_ptr(), result);
			ArwenBigUint { handle: result }
		}
	}


	#[inline]
	fn get_call_value_big_uint(&self) -> ArwenBigUint {
		unsafe {
			let result = bigIntNew(0);
			bigIntGetCallValue(result);
			ArwenBigUint { handle: result }
		}
	}

	#[inline]
	fn get_esdt_value_big_uint(&self) -> ArwenBigUint {
		unsafe {
			let result = bigIntNew(0);
			bigIntGetESDTCallValue(result);
			ArwenBigUint { handle: result }
		}
	}

	fn get_esdt_token_name(&self) -> Vec<u8> {
		// TODO: returning a boxed slice instead should marginally improve performance
		unsafe {
			let mut name = Vec::with_capacity(32);
			let name_len = getESDTTokenName(name.as_mut_ptr());
			name.set_len(name_len as usize);
			name
		}
	}

	fn send_tx(&self, to: &Address, amount: &ArwenBigUint, data: &[u8]) {
		let amount_bytes32 = amount.to_bytes_be_pad_right(32).unwrap(); // TODO: unwrap panics, remove
		unsafe {
			transferValue(
				to.as_ref().as_ptr(),
				amount_bytes32.as_ptr(),
				data.as_ptr(),
				data.len() as i32,
			);
		}
	}

	fn async_call(&self, to: &Address, amount: &ArwenBigUint, data: &[u8]) {
		let amount_bytes32 = amount.to_bytes_be_pad_right(32).unwrap(); // TODO: unwrap panics, remove
		unsafe {
			asyncCall(
				to.as_ref().as_ptr(),
				amount_bytes32.as_ptr(),
				data.as_ptr(),
				data.len() as i32,
			);
		}
	}

	fn deploy_contract(
		&self,
		gas: u64,
		amount: &ArwenBigUint,
		code: &BoxedBytes,
		code_metadata: CodeMetadata,
		arg_buffer: &ArgBuffer,
	) -> Address {
		let amount_bytes32 = amount.to_bytes_be_pad_right(32).unwrap(); // TODO: unwrap panics, remove
		let mut new_address = Address::zero();
		unsafe {
			let _ = createContract(
				gas,
				amount_bytes32.as_ptr(),
				code.as_ptr(),
				code_metadata.as_ptr(),
				code.len() as i32,
				new_address.as_mut_ptr(),
				arg_buffer.num_args() as i32,
				arg_buffer.arg_lengths_bytes_ptr(),
				arg_buffer.arg_data_ptr(),
			);
		}
		new_address
	}

	#[inline]
	fn get_tx_hash(&self) -> H256 {
		unsafe {
			let mut res = H256::zero();
			getOriginalTxHash(res.as_mut_ptr());
			res.into()
		}
	}

	#[inline]
	fn get_gas_left(&self) -> u64 {
		unsafe { getGasLeft() as u64 }
	}

	#[inline]
	fn get_block_timestamp(&self) -> u64 {
		unsafe { getBlockTimestamp() as u64 }
	}

	#[inline]
	fn get_block_nonce(&self) -> u64 {
		unsafe { getBlockNonce() as u64 }
	}

	#[inline]
	fn get_block_round(&self) -> u64 {
		unsafe { getBlockRound() as u64 }
	}

	#[inline]
	fn get_block_epoch(&self) -> u64 {
		unsafe { getBlockEpoch() as u64 }
	}

	#[inline]
	fn get_block_random_seed(&self) -> Box<[u8; 48]> {
		unsafe {
			let mut res = [0u8; 48];
			getBlockRandomSeed(res.as_mut_ptr());
			Box::new(res)
		}
	}

	#[inline]
	fn get_prev_block_timestamp(&self) -> u64 {
		unsafe { getPrevBlockTimestamp() as u64 }
	}

	#[inline]
	fn get_prev_block_nonce(&self) -> u64 {
		unsafe { getPrevBlockNonce() as u64 }
	}

	#[inline]
	fn get_prev_block_round(&self) -> u64 {
		unsafe { getPrevBlockRound() as u64 }
	}

	#[inline]
	fn get_prev_block_epoch(&self) -> u64 {
		unsafe { getPrevBlockEpoch() as u64 }
	}

	#[inline]
	fn get_prev_block_random_seed(&self) -> Box<[u8; 48]> {
		unsafe {
			let mut res = [0u8; 48];
			getPrevBlockRandomSeed(res.as_mut_ptr());
			Box::new(res)
		}
	}

	fn sha256(&self, data: &[u8]) -> H256 {
		unsafe {
			let mut res = H256::zero();
			sha256(data.as_ptr(), data.len() as i32, res.as_mut_ptr());
			res
		}
	}

	fn keccak256(&self, data: &[u8]) -> H256 {
		unsafe {
			let mut res = H256::zero();
			keccak256(data.as_ptr(), data.len() as i32, res.as_mut_ptr());
			res
		}
	}
}

impl elrond_wasm::ContractIOApi<ArwenBigInt, ArwenBigUint> for ArwenApiImpl {
	#[inline]
	fn get_num_arguments(&self) -> i32 {
		unsafe { getNumArguments() }
	}

	fn check_not_payable(&self) {
		if self.get_call_value_big_uint() > 0 {
			self.signal_error(err_msg::NON_PAYABLE);
		}
	}

	#[inline]
	fn get_argument_len(&self, arg_index: i32) -> usize {
		unsafe { getArgumentLength(arg_index) as usize }
	}

	fn copy_argument_to_slice(&self, arg_index: i32, slice: &mut [u8]) {
		unsafe {
			let byte_len = getArgument(arg_index, slice.as_mut_ptr()) as usize;
			if byte_len != slice.len() {
				self.signal_error(err_msg::ARG_BAD_LENGTH);
			}
		}
	}

	fn get_argument_vec_u8(&self, arg_index: i32) -> Vec<u8> {
		let len = self.get_argument_len(arg_index);
		let mut res = Vec::with_capacity(len);
		if len > 0 {
			unsafe {
				res.set_len(len);
				getArgument(arg_index, res.as_mut_ptr());
			}
		}
		res
	}

	fn get_argument_boxed_bytes(&self, arg_index: i32) -> BoxedBytes {
		let len = self.get_argument_len(arg_index);
		unsafe {
			let mut res = BoxedBytes::allocate(len);
			if len > 0 {
				getArgument(arg_index, res.as_mut_ptr());
			}
			res
		}
	}

	#[inline]
	fn get_argument_big_uint(&self, arg_id: i32) -> ArwenBigUint {
		ArwenBigUint {
			handle: self.get_argument_big_uint_raw(arg_id),
		}
	}

	#[inline]
	fn get_argument_big_int(&self, arg_id: i32) -> ArwenBigInt {
		ArwenBigInt {
			handle: self.get_argument_big_int_raw(arg_id),
		}
	}

	fn get_argument_big_uint_raw(&self, arg_id: i32) -> i32 {
		unsafe {
			let handle = bigIntNew(0);
			bigIntGetUnsignedArgument(arg_id, handle);
			handle
		}
	}

	fn get_argument_big_int_raw(&self, arg_id: i32) -> i32 {
		unsafe {
			let handle = bigIntNew(0);
			bigIntGetSignedArgument(arg_id, handle);
			handle
		}
	}

	#[inline]
	fn get_argument_u64(&self, arg_id: i32) -> u64 {
		unsafe { smallIntGetUnsignedArgument(arg_id) as u64 }
	}

	#[inline]
	fn get_argument_i64(&self, arg_id: i32) -> i64 {
		unsafe { smallIntGetSignedArgument(arg_id) }
	}

	#[inline]
	fn finish_slice_u8(&self, slice: &[u8]) {
		unsafe {
			finish(slice.as_ptr(), slice.len() as i32);
		}
	}

	#[inline]
	fn finish_big_int(&self, b: &ArwenBigInt) {
		unsafe {
			bigIntFinishSigned(b.handle);
		}
	}

	#[inline]
	fn finish_big_uint(&self, b: &ArwenBigUint) {
		unsafe {
			bigIntFinishUnsigned(b.handle);
		}
	}

	#[inline]
	fn finish_big_int_raw(&self, handle: i32) {
		unsafe {
			bigIntFinishSigned(handle);
		}
	}

	#[inline]
	fn finish_big_uint_raw(&self, handle: i32) {
		unsafe {
			bigIntFinishUnsigned(handle);
		}
	}

	#[inline]
	fn finish_u64(&self, value: u64) {
		unsafe {
			smallIntFinishUnsigned(value as i64);
		}
	}

	#[inline]
	fn finish_i64(&self, value: i64) {
		unsafe {
			smallIntFinishSigned(value);
		}
	}

	// #[inline]
	// fn signal_error(&self, message: &[u8]) -> ! {
	// 	ext_error::signal_error(message)
	// }

	fn write_log(&self, topics: &[[u8; 32]], data: &[u8]) {
		let mut topics_raw = [0u8; TOPIC_LENGTH * 10]; // hopefully we never have more than 10 topics
		for i in 0..topics.len() {
			topics_raw[TOPIC_LENGTH * i..TOPIC_LENGTH * (i + 1)].copy_from_slice(&topics[i]);
		}
		unsafe {
			writeLog(
				data.as_ptr(),
				data.len() as i32,
				topics_raw.as_ptr(),
				topics.len() as i32,
			);
		}
	}
}

/// Should be no-op. The API implementation is zero-sized.
impl Clone for ArwenApiImpl {
	#[inline]
	fn clone(&self) -> Self {
		ArwenApiImpl {}
	}
}
