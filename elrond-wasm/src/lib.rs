#![no_std]

// re-export basic heap types
extern crate alloc;
pub use alloc::boxed::Box;
pub use alloc::string::String;
pub use alloc::vec::Vec;

pub use elrond_codec;

pub mod abi;
pub mod api;
pub mod err_msg;
pub mod hex_call_data;
pub mod io;
pub mod non_zero_util;
mod proxy;
pub mod storage;
pub mod types;

use api::{
	CryptoApi, EndpointArgumentApi, EndpointFinishApi, ErrorApi, LogApi, StorageReadApi,
	StorageWriteApi,
};
pub use hex_call_data::*;
pub use io::*;
pub use proxy::OtherContractHandle;
pub use storage::{storage_get, storage_set};
pub use types::*;

use core::ops::{Add, Div, Mul, Neg, Rem, Sub};
use core::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};
use core::ops::{BitAnd, BitOr, BitXor, Shl, Shr};
use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign};

/// Interface to be used by the actual smart contract code.
///
/// Note: contracts and the api are not mutable.
/// They simply pass on/retrieve data to/from the protocol.
/// When mocking the blockchain state, we use the Rc/RefCell pattern
/// to isolate mock state mutability from the contract interface.
pub trait ContractHookApi<BigInt, BigUint>:
	Sized + StorageReadApi + StorageWriteApi + ErrorApi + CryptoApi
// pub trait ContractHookApi<BigInt, BigUint, StorageRead, StorageWrite, Crypto>: Sized
where
	BigInt: elrond_codec::NestedEncode + 'static,
	BigUint: elrond_codec::NestedEncode + 'static,
{
	fn get_sc_address(&self) -> Address;

	fn get_owner_address(&self) -> Address;

	fn get_caller(&self) -> Address;

	fn get_balance(&self, address: &Address) -> BigUint;

	fn get_sc_balance(&self) -> BigUint {
		self.get_balance(&self.get_sc_address())
	}

	// #[inline]
	// fn storage_load_cumulated_validator_reward(&self) -> BigUint {
	// 	self.storage_load_big_uint(storage::protected_keys::ELROND_REWARD_KEY)
	// }

	fn get_call_value_big_uint(&self) -> BigUint;

	fn get_esdt_value_big_uint(&self) -> BigUint;

	fn get_esdt_token_name(&self) -> Vec<u8>;

	fn send_tx(&self, to: &Address, amount: &BigUint, data: &[u8]);

	fn async_call(&self, to: &Address, amount: &BigUint, data: &[u8]);

	fn deploy_contract(
		&self,
		gas: u64,
		amount: &BigUint,
		code: &BoxedBytes,
		code_metadata: CodeMetadata,
		arg_buffer: &ArgBuffer,
	) -> Address;

	fn get_tx_hash(&self) -> H256;

	fn get_gas_left(&self) -> u64;

	fn get_block_timestamp(&self) -> u64;

	fn get_block_nonce(&self) -> u64;

	fn get_block_round(&self) -> u64;

	fn get_block_epoch(&self) -> u64;

	fn get_block_random_seed(&self) -> Box<[u8; 48]>;

	fn get_prev_block_timestamp(&self) -> u64;

	fn get_prev_block_nonce(&self) -> u64;

	fn get_prev_block_round(&self) -> u64;

	fn get_prev_block_epoch(&self) -> u64;

	fn get_prev_block_random_seed(&self) -> Box<[u8; 48]>;

	fn execute_on_dest_context(
		&self,
		gas: u64,
		address: &Address,
		value: &BigUint,
		function: &[u8],
		arg_buffer: &ArgBuffer,
	);

	fn execute_on_dest_context_by_caller(
		&self,
		gas: u64,
		address: &Address,
		value: &BigUint,
		function: &[u8],
		arg_buffer: &ArgBuffer,
	);

	fn execute_on_same_context(
		&self,
		gas: u64,
		address: &Address,
		value: &BigUint,
		function: &[u8],
		arg_buffer: &ArgBuffer,
	);

	// fn storage_read_raw_api(&self) -> StorageRead;

	// fn storage_write_raw_api(&self) -> StorageWrite;

	// fn crypto(&self) -> Crypto;
}

pub trait ContractIOApi<BigInt, BigUint>:
	Clone + ErrorApi + EndpointArgumentApi + EndpointFinishApi + LogApi
{
}

/// Definition of the BigUint type required by the API.
/// The API doesn't care about the actual BigInt implementation.
/// The Arwen VM provides an implementation directly in the protocol.
/// For debugging we use a different implementation, based on Rust's BigInt.
///
/// Since most values in smart contracts will not be signed, as well as for safety,
/// most of the functionality if provided for unsigned integers.
pub trait BigUintApi:
	Sized
	+ From<u64>
	+ From<u32>
	+ From<usize>
	+ Clone
	+ Add<Output = Self>
	+ AddAssign
	+ Sub<Output = Self>
	+ SubAssign
	+ Mul<Output = Self>
	+ MulAssign
	+ Div<Output = Self>
	+ DivAssign
	+ Rem<Output = Self>
	+ RemAssign
	+ BitAnd<Output = Self>
	+ BitAndAssign
	+ BitOr<Output = Self>
	+ BitOrAssign
	+ BitXor<Output = Self>
	+ BitXorAssign
	+ Shr<usize, Output = Self>
	+ ShrAssign<usize>
	+ Shl<usize, Output = Self>
	+ ShlAssign<usize>
	+ PartialEq<Self>
	+ Eq
	+ PartialOrd<Self>
	+ Ord
	+ PartialEq<u64>
	+ PartialOrd<u64>
	+ elrond_codec::NestedEncode
	+ elrond_codec::TopEncode
	+ elrond_codec::NestedDecode
	+ elrond_codec::TopDecode
	+ abi::TypeAbi
{
	fn zero() -> Self {
		0u64.into()
	}

	fn byte_length(&self) -> i32;

	fn copy_to_slice_big_endian(&self, slice: &mut [u8]) -> i32;

	fn copy_to_array_big_endian_pad_right(&self, target: &mut [u8; 32]);

	fn to_bytes_be(&self) -> Vec<u8>;

	fn to_bytes_be_pad_right(&self, nr_bytes: usize) -> Option<Vec<u8>>;

	fn from_bytes_be(bytes: &[u8]) -> Self;
}

// BigInt sign.
pub enum Sign {
	Minus,
	NoSign,
	Plus,
}

/// Definition of the BigInt type required by the API.
pub trait BigIntApi<BigUint>:
	Sized
	+ From<BigUint>
	+ From<i64>
	+ From<i32>
	+ Clone
	+ Add<Output = Self>
	+ AddAssign
	+ Sub<Output = Self>
	+ SubAssign
	+ Mul<Output = Self>
	+ MulAssign
	+ Div<Output = Self>
	+ DivAssign
	+ Rem<Output = Self>
	+ RemAssign
	+ Neg
	+ PartialEq<Self>
	+ Eq
	+ PartialOrd<Self>
	+ Ord
	+ PartialEq<i64>
	+ PartialOrd<i64>
	+ elrond_codec::NestedEncode
	+ elrond_codec::TopEncode
	+ elrond_codec::NestedDecode
	+ elrond_codec::TopDecode
	+ abi::TypeAbi
{
	fn zero() -> Self {
		0i64.into()
	}

	fn abs_uint(&self) -> BigUint;

	fn sign(&self) -> Sign;

	fn to_signed_bytes_be(&self) -> Vec<u8>;

	fn from_signed_bytes_be(bytes: &[u8]) -> Self;
}

/// CallableContract is the means by which the debugger calls methods in the contract.
pub trait CallableContract<A> {
	fn call(&self, fn_name: &[u8]) -> bool;

	fn abi(&self, include_modules: bool) -> abi::ContractAbi;

	fn clone_contract(&self) -> Box<dyn CallableContract<A>>;

	fn into_api(self: Box<Self>) -> A;
}

/// Handy way of casting to a contract proxy trait.
/// Would make more sense to be in elrond-wasm-derive, but Rust "cannot export macro_rules! macros from a `proc-macro` crate type currently".
#[macro_export]
macro_rules! contract_proxy {
	($s:expr, $address:expr, $proxy_trait:ident) => {
		$s.contract_proxy($address) as Box<dyn $proxy_trait<BigInt, BigUint>>
	};
}

/// Getting all imports needed for a smart contract.
#[macro_export]
macro_rules! imports {
	() => {
		use core::ops::{Add, Div, Mul, Rem, Sub};
		use core::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};
		use core::ops::{BitAnd, BitOr, BitXor, Shl, Shr};
		use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign};
		use elrond_wasm::elrond_codec::{DecodeError, NestedDecode, NestedEncode, TopDecode};
		use elrond_wasm::err_msg;
		use elrond_wasm::io::*;
		use elrond_wasm::non_zero_util::*;
		use elrond_wasm::types::*;
		use elrond_wasm::{Address, H256};
		use elrond_wasm::{
			AsyncCallError, AsyncCallResult, BigIntApi, BigUintApi, ContractHookApi, ContractIOApi,
			OtherContractHandle,
		};
		use elrond_wasm::{BorrowedMutStorage, Box, BoxedBytes, Queue, VarArgs, Vec};
		use elrond_wasm::{SCError, SCResult, SCResult::Err, SCResult::Ok};
	};
}

/// Imports required for deriving serialization and TypeAbi.
#[macro_export]
macro_rules! derive_imports {
	() => {
		use elrond_wasm::elrond_codec;
		use elrond_wasm::elrond_codec::elrond_codec_derive::{
			NestedDecode, NestedEncode, TopDecode, TopEncode,
		};
		use elrond_wasm_derive::TypeAbi;
	};
}

/// Compact way of returning a static error message.
#[macro_export]
macro_rules! sc_error {
	($s:expr) => {
		elrond_wasm::SCResult::Err(elrond_wasm::SCError::from($s.as_bytes()))
	};
}

/// Equivalent of the ? operator for SCResult.
#[macro_export]
macro_rules! sc_try {
	($s:expr) => {
		match $s {
			elrond_wasm::SCResult::Ok(t) => t,
			elrond_wasm::SCResult::Err(e) => {
				return elrond_wasm::SCResult::Err(e);
			},
		}
	};
}

/// Allows us to write Solidity style `require!(<condition>, <error_msg>)` and avoid if statements.
///
/// It can only be used in a function that returns `SCResult<_>` where _ can be any type.
///
/// ```rust
/// # use elrond_wasm::{*, SCResult::Ok};
/// # pub trait ExampleContract<BigInt, BigUint>: ContractHookApi<BigInt, BigUint>
/// # where
/// #     BigInt: elrond_codec::NestedEncode + 'static,
/// #     BigUint: elrond_codec::NestedEncode + 'static,
/// # {
/// fn only_callable_by_owner(&self) -> SCResult<()> {
///     require!(self.get_caller() == self.get_owner_address(), "Caller must be owner");
///     Ok(())
/// }
/// # }
/// ```
#[macro_export]
macro_rules! require {
	($expression:expr, $error_msg:expr) => {
		if (!($expression)) {
			return sc_error!($error_msg);
		}
	};
}

/// Very compact way of not allowing anyone but the owner to call a function.
///
/// It can only be used in a function that returns `SCResult<_>` where _ can be any type.
///
/// ```rust
/// # use elrond_wasm::{*, SCResult::Ok};
/// # pub trait ExampleContract<BigInt, BigUint>: ContractHookApi<BigInt, BigUint>
/// # where
/// #     BigInt: elrond_codec::NestedEncode + 'static,
/// #     BigUint: elrond_codec::NestedEncode + 'static,
/// # {
/// fn only_callable_by_owner(&self) -> SCResult<()> {
///     only_owner!(self, "Caller must be owner");
///     Ok(())
/// }
/// # }
/// ```
#[macro_export]
macro_rules! only_owner {
	($trait_self: expr, $error_msg:expr) => {
		if ($trait_self.get_caller() != $trait_self.get_owner_address()) {
			return sc_error!($error_msg);
		}
	};
}

/// Compact way to represent the BorrowedMutStorage type.
#[macro_export]
macro_rules! mut_storage (
    ($t:ty) => (
        BorrowedMutStorage<T, $t>
    )
);

/// Converts usize to NonZeroUsize or returns SCError.
#[macro_export]
macro_rules! non_zero_usize {
	($input: expr, $error_msg:expr) => {
		if let Some(nz) = NonZeroUsize::new($input) {
			nz
		} else {
			return sc_error!($error_msg);
		}
	};
}
