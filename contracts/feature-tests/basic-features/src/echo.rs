elrond_wasm::imports!();

use crate::types::*;
use core::num::NonZeroUsize;

/// Test endpoint argument and result serialization.
#[elrond_wasm::module]
pub trait EchoTypes {
    #[endpoint]
    fn echo_u64(&self, i: u64) -> u64 {
        i
    }

    #[endpoint]
    fn echo_i64(&self, i: i64) -> i64 {
        i
    }

    #[endpoint]
    fn echo_i32(&self, i: i32) -> i32 {
        i
    }

    #[endpoint]
    fn echo_u32(&self, i: u32) -> u32 {
        i
    }

    #[endpoint]
    fn echo_isize(&self, i: isize) -> isize {
        i
    }

    #[endpoint]
    fn echo_usize(&self, i: usize) -> usize {
        i
    }

    #[endpoint]
    fn echo_i8(&self, i: i8) -> i8 {
        i
    }

    #[endpoint]
    fn echo_u8(&self, i: u8) -> u8 {
        i
    }

    #[endpoint]
    fn echo_bool(&self, i: bool) -> bool {
        i
    }

    #[endpoint]
    fn echo_opt_bool(&self, i: Option<bool>) -> Option<bool> {
        i
    }

    #[endpoint]
    fn echo_nothing(&self, #[var_args] nothing: ()) -> () {
        nothing
    }

    #[endpoint]
    fn echo_array_u8(&self, s: [u8; 5]) -> [u8; 5] {
        s
    }

    #[endpoint]
    fn echo_multi_value_u32(
        &self,
        #[var_args] m: MultiValueManagedVec<u32>,
    ) -> MultiValue2<usize, MultiValueManagedVec<u32>> {
        let v = m.into_vec();
        (v.len(), v.into()).into()
    }

    #[endpoint]
    fn echo_multi_value_tuples(
        &self,
        #[var_args] m: MultiValueVec<MultiValue2<isize, Vec<u8>>>,
    ) -> MultiValueVec<MultiValue2<isize, Vec<u8>>> {
        let mut result: Vec<MultiValue2<isize, Vec<u8>>> = Vec::new();
        for m_arg in m.into_vec().into_iter() {
            result.push(m_arg.into_tuple().into())
        }
        result.into()
    }

    #[endpoint]
    fn echo_ser_example_2(&self, se: ExampleEnumWithFields) -> ExampleEnumWithFields {
        se
    }

    #[view]
    fn echo_simple_enum(&self, se: ExampleEnumSimple) -> ExampleEnumSimple {
        se
    }

    #[view]
    fn finish_simple_enum_variant_1(&self) -> ExampleEnumSimple {
        ExampleEnumSimple::Variant1
    }

    #[view]
    fn echo_non_zero_usize(&self, nz: NonZeroUsize) -> NonZeroUsize {
        nz
    }

    #[view]
    fn echo_some_args_ignore_others(
        &self,
        i: i32,
        #[var_args] opt: OptionalValue<i32>,
        #[var_args] _ignore: IgnoreValue,
    ) -> MultiValue2<i32, OptionalValue<i32>> {
        (i, opt).into()
    }

    #[view]
    fn echo_arrayvec(&self, av: ArrayVec<i32, 3>) -> ArrayVec<i32, 3> {
        av
    }
}
