use crate::{abi::TypeAbi, types::BoxedBytes, ContractCallArg, DynArgOutput};
use alloc::string::String;
use elrond_codec::{DecodeErrorHandler, TopDecodeMulti, TopDecodeMultiInput};

pub struct AsyncCallError {
    pub err_code: u32,
    pub err_msg: BoxedBytes,
}

pub enum AsyncCallResult<T> {
    Ok(T),
    Err(AsyncCallError),
}

impl<T> AsyncCallResult<T> {
    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(self, AsyncCallResult::Ok(_))
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

impl<T> TopDecodeMulti for AsyncCallResult<T>
where
    T: TopDecodeMulti,
{
    fn multi_decode_or_handle_err<I, H>(input: &mut I, h: H) -> Result<Self, H::HandledErr>
    where
        I: TopDecodeMultiInput,
        H: DecodeErrorHandler,
    {
        let err_code: u32 = input.next_value(h)?;
        if err_code == 0 {
            Ok(Self::Ok(T::multi_decode_or_handle_err(input, h)?))
        } else {
            let err_msg = if input.has_next() {
                input.next_value(h)?
            } else {
                // temporary fix, until a problem involving missing error messages in the protocol gets fixed
                // can be removed after the protocol is patched
                // error messages should not normally be missing
                BoxedBytes::empty()
            };
            Ok(Self::Err(AsyncCallError { err_code, err_msg }))
        }
    }
}

impl<T> ContractCallArg for &AsyncCallResult<T>
where
    T: ContractCallArg,
{
    fn push_dyn_arg<O: DynArgOutput>(&self, output: &mut O) {
        match self {
            AsyncCallResult::Ok(result) => {
                0u32.push_dyn_arg(output);
                result.push_dyn_arg(output);
            },
            AsyncCallResult::Err(error_message) => {
                error_message.err_code.push_dyn_arg(output);
                error_message.err_msg.push_dyn_arg(output);
            },
        }
    }
}

impl<T> ContractCallArg for AsyncCallResult<T>
where
    T: ContractCallArg,
{
    fn push_dyn_arg<O: DynArgOutput>(&self, output: &mut O) {
        ContractCallArg::push_dyn_arg(&self, output)
    }
}

impl<T: TypeAbi> TypeAbi for AsyncCallResult<T> {
    fn type_name() -> String {
        let mut repr = String::from("AsyncCallResult<");
        repr.push_str(T::type_name().as_str());
        repr.push('>');
        repr
    }
}
