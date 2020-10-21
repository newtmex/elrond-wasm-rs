#![no_std]

// re-export basic heap types
extern crate alloc;
pub use alloc::boxed::Box;
pub use alloc::vec::Vec;
pub use alloc::string::String;

pub use elrond_codec;

mod types;
pub mod io;
mod proxy;
pub mod storage;
pub mod err_msg;
pub mod call_data;
pub mod non_zero_util;

pub use types::*;
pub use io::*;
pub use storage::{storage_get, storage_set, BorrowedMutStorage};
pub use io::SCResult;
pub use call_data::*;
pub use proxy::OtherContractHandle;

use core::ops::{Add, Sub, Mul, Div, Rem, Neg};
use core::ops::{AddAssign, SubAssign, MulAssign, DivAssign, RemAssign};
use core::ops::{BitAnd, BitOr, BitXor, Shr, Shl};
use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign, ShrAssign, ShlAssign};

/// Interface to be used by the actual smart contract code.
/// 
/// Note: contracts and the api are not mutable.
/// They simply pass on/retrieve data to/from the protocol.
/// When mocking the blockchain state, we use the Rc/RefCell pattern 
/// to isolate mock state mutability from the contract interface.
pub trait ContractHookApi<BigInt, BigUint>: Sized
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
    
    fn storage_store_slice_u8(&self, key: &[u8], value: &[u8]);

    fn storage_load_len(&self, key: &[u8]) -> usize;

    fn storage_load_vec_u8(&self, key: &[u8]) -> Vec<u8>;

    fn storage_load_boxed_slice_u8(&self, key: &[u8]) -> Box<[u8]> {
        self.storage_load_vec_u8(key).into_boxed_slice()
    }

    fn storage_store_bytes32(&self, key: &[u8], value: &[u8; 32]);
    
    fn storage_load_bytes32(&self, key: &[u8]) -> [u8; 32];

    fn storage_store_big_uint(&self, key: &[u8], value: &BigUint);
    
    fn storage_load_big_uint(&self, key: &[u8]) -> BigUint;

    fn storage_store_big_int(&self, key: &[u8], value: &BigInt);
    
    fn storage_load_big_int(&self, key: &[u8]) -> BigInt;

    fn storage_store_u64(&self, key: &[u8], value: u64);

    fn storage_store_i64(&self, key: &[u8], value: i64);

    fn storage_load_u64(&self, key: &[u8]) -> u64;

    fn storage_load_i64(&self, key: &[u8]) -> i64;

    #[inline]
    fn storage_load_cumulated_validator_reward(&self) -> BigUint {
        self.storage_load_big_uint(storage::protected_keys::ELROND_REWARD_KEY)
    }
    
    fn get_call_value_big_uint(&self) -> BigUint;

    fn get_esdt_value_big_uint(&self) -> BigUint;

    fn get_esdt_token_name(&self) -> Option<Vec<u8>>;

    fn send_tx(&self, to: &Address, amount: &BigUint, message: &str);

    fn async_call(&self, to: &Address, amount: &BigUint, data: &[u8]);

    fn get_tx_hash(&self) -> H256;

    fn get_gas_left(&self) -> u64;

    fn get_block_timestamp(&self) -> u64;

    fn get_block_nonce(&self) -> u64;

    fn get_block_round(&self) -> u64;
    
    fn get_block_epoch(&self) -> u64;

    fn get_prev_block_timestamp(&self) -> u64;

    fn get_prev_block_nonce(&self) -> u64;

    fn get_prev_block_round(&self) -> u64;

    fn get_prev_block_epoch(&self) -> u64;

    fn sha256(&self, data: &[u8]) -> H256;

    fn keccak256(&self, data: &[u8]) -> H256;
}

/// Interface to only be used by code generated by the macros.
/// The smart contract code doesn't have access to these methods directly.
pub trait ContractIOApi<BigInt, BigUint> {

    fn get_num_arguments(&self) -> i32;

    #[inline(never)] // prevent inline intentinally
    fn check_num_arguments(&self, expected: i32) {
        let nr_args = self.get_num_arguments();
        if nr_args != expected {
            self.signal_error(err_msg::ARG_WRONG_NUMBER);
        }
    }

    fn check_not_payable(&self);

    fn get_argument_len(&self, arg_index: i32) -> usize;

    fn copy_argument_to_slice(&self, arg_index: i32, slice: &mut [u8]);

    fn get_argument_vec_u8(&self, arg_index: i32) -> Vec<u8>;

    fn get_argument_boxed_slice_u8(&self, arg_index: i32) -> Box<[u8]> {
        self.get_argument_vec_u8(arg_index).into_boxed_slice()
    }

    fn get_argument_bytes32(&self, arg_index: i32) -> [u8; 32];
    
    fn get_argument_address(&self, arg_index: i32) -> Address {
        self.get_argument_bytes32(arg_index).into()
    }
    
    fn get_argument_big_int(&self, arg_id: i32) -> BigInt;

    fn get_argument_big_uint(&self, arg_id: i32) -> BigUint;

    fn get_argument_u64(&self, arg_id: i32) -> u64;

    fn get_argument_i64(&self, arg_id: i32) -> i64;
    
    fn finish_slice_u8(&self, slice: &[u8]);

    fn finish_bytes32(&self, bytes: &[u8; 32]);

    fn finish_big_int(&self, b: &BigInt);

    fn finish_big_uint(&self, b: &BigUint);

    fn finish_u64(&self, value: u64);

    fn finish_i64(&self, value: i64);

    fn signal_error(&self, message: &[u8]) -> !;

    fn write_log(&self, topics: &[[u8;32]], data: &[u8]);
}

/// Definition of the BigUint type required by the API.
/// The API doesn't care about the actual BigInt implementation.
/// The Arwen VM provides an implementation directly in the protocol.
/// For debugging we use a different implementation, based on Rust's BigInt.
/// 
/// Since most values in smart contracts will not be signed, as well as for safety,
/// most of the functionality if provided for unsigned integers.
pub trait BigUintApi: 
    Sized +
    From<u64> +
    From<u32> +
    From<usize> +
    Clone +
    Add<Output=Self> + 
    AddAssign + 
    Sub<Output=Self> + 
    SubAssign +
    Mul<Output=Self> +
    MulAssign +
    Div<Output=Self> +
    DivAssign +
    Rem<Output=Self> +
    RemAssign +
    BitAnd<Output=Self> +
    BitAndAssign +
    BitOr<Output=Self> +
    BitOrAssign +
    BitXor<Output=Self> +
    BitXorAssign +
    Shr<usize, Output=Self> +
    ShrAssign<usize> +
    Shl<usize, Output=Self> +
    ShlAssign<usize> +
    PartialEq<Self> +
    Eq +
    PartialOrd<Self> +
    Ord +
    PartialEq<u64> +
    PartialOrd<u64> +
    elrond_codec::NestedEncode +
    elrond_codec::TopEncode +
    elrond_codec::NestedDecode +
    elrond_codec::TopDecode +
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
        Sized +
        From<BigUint> +
        From<i64> +
        From<i32> +
        Clone +
        Add<Output=Self> + 
        AddAssign + 
        Sub<Output=Self> + 
        SubAssign +
        Mul<Output=Self> +
        MulAssign +
        Div<Output=Self> +
        DivAssign +
        Rem<Output=Self> +
        RemAssign +
        Neg +
        PartialEq<Self> +
        Eq +
        PartialOrd<Self> +
        Ord +
        PartialEq<i64> +
        PartialOrd<i64> +
        elrond_codec::NestedEncode +
        elrond_codec::TopEncode +
        elrond_codec::NestedDecode +
        elrond_codec::TopDecode +
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
        use elrond_wasm::{Box, Vec, String, Queue, VarArgs, BorrowedMutStorage};
        use elrond_wasm::{SCError, SCResult, SCResult::Ok, SCResult::Err};
        use elrond_wasm::{H256, Address};
        use elrond_wasm::{ContractHookApi, ContractIOApi, BigIntApi, BigUintApi, OtherContractHandle, AsyncCallResult, AsyncCallError};
        use elrond_wasm::elrond_codec::{NestedEncode, NestedDecode, TopDecode, DecodeError};
        use elrond_wasm::io::*;
        use elrond_wasm::non_zero_util::*;
        use elrond_wasm::err_msg;
        use core::ops::{Add, Sub, Mul, Div, Rem};
        use core::ops::{AddAssign, SubAssign, MulAssign, DivAssign, RemAssign};
        use core::ops::{BitAnd, BitOr, BitXor, Shr, Shl};
        use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign, ShrAssign, ShlAssign};
  };
}

/// Compact way of returning a static error message.
#[macro_export]
macro_rules! sc_error {
    ($s:expr) => {
        elrond_wasm::SCResult::Err(elrond_wasm::SCError::from($s.as_bytes()))
    }
}

/// Equivalent of the ? operator for SCResult.
#[macro_export]
macro_rules! sc_try {
    ($s:expr) => {
        match $s {
            elrond_wasm::SCResult::Ok(t) => t,
            elrond_wasm::SCResult::Err(e) => { return elrond_wasm::SCResult::Err(e); }
        }
        
    }
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
			return sc_error!($error_msg)
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
			return sc_error!($error_msg)
		}
	};
}

/// Compact way to represent the BorrowedMutStorage type.
#[macro_export]
macro_rules! mut_storage (
    ($t:ty) => (
        BorrowedMutStorage<'_, T, BigInt, BigUint, $t>
    )
);

/// Converts usize to NonZeroUsize or returns SCError.
#[macro_export]
macro_rules! non_zero_usize {
	($input: expr, $error_msg:expr) => {
        if let Some(nz) = NonZeroUsize::new($input) {
            nz
        } else {
			return sc_error!($error_msg)
		}
	};
}
