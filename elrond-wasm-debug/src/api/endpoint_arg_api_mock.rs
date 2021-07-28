use crate::{TxContext, TxPanic};
use alloc::vec::Vec;
use elrond_wasm::api::EndpointArgumentApi;
use elrond_wasm::types::BoxedBytes;
use num_bigint::{BigInt, BigUint};
use num_traits::cast::ToPrimitive;

/// Interface to only be used by code generated by the macros.
/// The smart contract code doesn't have access to these methods directly.
impl EndpointArgumentApi for TxContext {
    fn get_num_arguments(&self) -> i32 {
        self.tx_input_box.args.len() as i32
    }

    fn get_argument_len(&self, arg_index: i32) -> usize {
        let arg = self.get_argument_vec_u8(arg_index);
        arg.len()
    }

    fn copy_argument_to_slice(&self, _arg_index: i32, _slice: &mut [u8]) {
        panic!("copy_argument_to_slice not yet implemented")
    }

    fn get_argument_vec_u8(&self, arg_index: i32) -> Vec<u8> {
        let arg_idx_usize = arg_index as usize;
        if arg_idx_usize >= self.tx_input_box.args.len() {
            panic!("Tx arg index out of range");
        }
        self.tx_input_box.args[arg_idx_usize].clone()
    }

    fn get_argument_boxed_bytes(&self, arg_index: i32) -> BoxedBytes {
        self.get_argument_vec_u8(arg_index).into()
    }

    fn get_argument_big_uint_raw(&self, _arg_index: i32) -> i32 {
        panic!("cannot call get_argument_big_uint_raw in debug mode");
    }

    fn get_argument_big_int_raw(&self, _arg_index: i32) -> i32 {
        panic!("cannot call get_argument_big_int_raw in debug mode");
    }

    fn get_argument_i64(&self, arg_index: i32) -> i64 {
        let bytes = self.get_argument_vec_u8(arg_index);
        let bi = BigInt::from_signed_bytes_be(&bytes);
        if let Some(v) = bi.to_i64() {
            v
        } else {
            std::panic::panic_any(TxPanic {
                status: 10,
                message: b"argument out of range".to_vec(),
            })
        }
    }

    fn get_argument_u64(&self, arg_index: i32) -> u64 {
        let bytes = self.get_argument_vec_u8(arg_index);
        let bu = BigUint::from_bytes_be(&bytes);
        if let Some(v) = bu.to_u64() {
            v
        } else {
            std::panic::panic_any(TxPanic {
                status: 10,
                message: b"argument out of range".to_vec(),
            })
        }
    }
}
