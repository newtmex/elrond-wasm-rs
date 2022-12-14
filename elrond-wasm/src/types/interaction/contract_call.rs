use elrond_codec::{TopDecodeMulti, TopEncodeMulti};

use crate::{
    api::{
        BlockchainApiImpl, CallTypeApi, ESDT_MULTI_TRANSFER_FUNC_NAME, ESDT_NFT_TRANSFER_FUNC_NAME,
        ESDT_TRANSFER_FUNC_NAME,
    },
    contract_base::{BlockchainWrapper, ExitCodecErrorHandler, SendRawWrapper},
    err_msg,
    io::{ArgErrorHandler, ArgId, ManagedResultArgLoader},
    types::{
        AsyncCall, BigUint, EgldOrEsdtTokenPayment, EsdtTokenPayment, ManagedAddress,
        ManagedArgBuffer, ManagedBuffer, ManagedVec,
    },
};
use core::marker::PhantomData;

use super::contract_call_common::{TRANSFER_EXECUTE_DEFAULT_LEFTOVER, UNSPECIFIED_GAS_LIMIT};

/// Represents metadata for calling another contract.
/// Can transform into either an async call, transfer call or other types of calls.
#[must_use]
pub struct ContractCall<SA, OriginalResult>
where
    SA: CallTypeApi + 'static,
{
    _phantom: PhantomData<SA>,
    pub to: ManagedAddress<SA>,
    pub egld_payment: BigUint<SA>,
    pub payments: ManagedVec<SA, EsdtTokenPayment<SA>>,
    pub endpoint_name: ManagedBuffer<SA>,
    pub explicit_gas_limit: u64,
    pub arg_buffer: ManagedArgBuffer<SA>,
    _return_type: PhantomData<OriginalResult>,
}

#[allow(clippy::return_self_not_must_use)]
impl<SA, OriginalResult> ContractCall<SA, OriginalResult>
where
    SA: CallTypeApi + 'static,
{
    pub fn new(to: ManagedAddress<SA>, endpoint_name: ManagedBuffer<SA>) -> Self {
        let payments = ManagedVec::new();
        Self::new_with_esdt_payment(to, endpoint_name, payments)
    }

    pub fn new_with_esdt_payment(
        to: ManagedAddress<SA>,
        endpoint_name: ManagedBuffer<SA>,
        payments: ManagedVec<SA, EsdtTokenPayment<SA>>,
    ) -> Self {
        let arg_buffer = ManagedArgBuffer::new();
        let egld_payment = BigUint::zero();
        ContractCall {
            _phantom: PhantomData,
            to,
            egld_payment,
            payments,
            explicit_gas_limit: UNSPECIFIED_GAS_LIMIT,
            endpoint_name,
            arg_buffer,
            _return_type: PhantomData,
        }
    }

    /// Adds a single ESDT token transfer to a contract call.
    ///
    /// Can be called multiple times on the same call.
    pub fn with_esdt_transfer<P: Into<EsdtTokenPayment<SA>>>(mut self, payment: P) -> Self {
        self.payments.push(payment.into());
        self
    }

    /// Sets payment to be either EGLD or a single ESDT transfer, as determined at runtime.
    pub fn with_egld_or_single_esdt_transfer<P: Into<EgldOrEsdtTokenPayment<SA>>>(
        self,
        payment: P,
    ) -> Self {
        let payment_cast = payment.into();
        if payment_cast.token_identifier.is_egld() {
            self.with_egld_transfer(payment_cast.amount)
        } else {
            self.with_esdt_transfer((
                payment_cast.token_identifier.unwrap_esdt(),
                payment_cast.token_nonce,
                payment_cast.amount,
            ))
        }
    }

    /// Sets payment to be EGLD transfer.
    pub fn with_egld_transfer(mut self, egld_amount: BigUint<SA>) -> Self {
        self.egld_payment = egld_amount;
        self
    }

    /// Sets payment to be a (potentially) multi-token transfer.
    #[inline]
    pub fn with_multi_token_transfer(
        mut self,
        payments: ManagedVec<SA, EsdtTokenPayment<SA>>,
    ) -> Self {
        self.payments = payments;
        self
    }

    #[inline]
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.explicit_gas_limit = gas_limit;
        self
    }

    pub fn with_arguments_raw(mut self, raw_argument_buffer: ManagedArgBuffer<SA>) -> Self {
        self.arg_buffer = raw_argument_buffer;
        self
    }

    /// Provided for cases where we build the contract call by hand.
    pub fn push_arg_managed_buffer(&mut self, m_buffer: ManagedBuffer<SA>) {
        self.arg_buffer.push_arg_raw(m_buffer)
    }

    pub fn push_endpoint_arg<T: TopEncodeMulti>(&mut self, endpoint_arg: &T) {
        let h = ExitCodecErrorHandler::<SA>::from(err_msg::CONTRACT_CALL_ENCODE_ERROR);
        let Ok(()) = endpoint_arg.multi_encode_or_handle_err(&mut self.arg_buffer, h);
    }

    fn no_payments(&self) -> ManagedVec<SA, EsdtTokenPayment<SA>> {
        ManagedVec::new()
    }

    /// If this is an ESDT call, it converts it to a regular call to ESDTTransfer.
    /// Async calls require this step, but not `transfer_esdt_execute`.
    pub fn convert_to_esdt_transfer_call(self) -> Self {
        match self.payments.len() {
            0 => self,
            1 => self.convert_to_single_transfer_esdt_call(),
            _ => self.convert_to_multi_transfer_esdt_call(),
        }
    }

    fn convert_to_single_transfer_esdt_call(self) -> Self {
        if let Some(payment) = self.payments.try_get(0) {
            if payment.token_nonce == 0 {
                let no_payments = self.no_payments();

                // fungible ESDT
                let mut new_arg_buffer = ManagedArgBuffer::new();
                new_arg_buffer.push_arg(&payment.token_identifier);
                new_arg_buffer.push_arg(&payment.amount);
                if !self.endpoint_name.is_empty() {
                    new_arg_buffer.push_arg(&self.endpoint_name);
                }

                let zero = BigUint::zero();
                let endpoint_name = ManagedBuffer::from(ESDT_TRANSFER_FUNC_NAME);

                ContractCall {
                    _phantom: PhantomData,
                    to: self.to,
                    egld_payment: zero,
                    payments: no_payments,
                    explicit_gas_limit: self.explicit_gas_limit,
                    endpoint_name,
                    arg_buffer: new_arg_buffer.concat(self.arg_buffer),
                    _return_type: PhantomData,
                }
            } else {
                let payments = self.no_payments();

                // NFT
                // `ESDTNFTTransfer` takes 4 arguments:
                // arg0 - token identifier
                // arg1 - nonce
                // arg2 - quantity to transfer
                // arg3 - destination address
                let mut new_arg_buffer = ManagedArgBuffer::new();
                new_arg_buffer.push_arg(&payment.token_identifier);
                new_arg_buffer.push_arg(payment.token_nonce);
                new_arg_buffer.push_arg(&payment.amount);
                new_arg_buffer.push_arg(&self.to);
                if !self.endpoint_name.is_empty() {
                    new_arg_buffer.push_arg(&self.endpoint_name);
                }

                // nft transfer is sent to self, sender = receiver
                let recipient_addr = BlockchainWrapper::<SA>::new().get_sc_address();
                let zero = BigUint::zero();
                let endpoint_name = ManagedBuffer::from(ESDT_NFT_TRANSFER_FUNC_NAME);

                ContractCall {
                    _phantom: PhantomData,
                    to: recipient_addr,
                    egld_payment: zero,
                    payments,
                    explicit_gas_limit: self.explicit_gas_limit,
                    endpoint_name,
                    arg_buffer: new_arg_buffer.concat(self.arg_buffer),
                    _return_type: PhantomData,
                }
            }
        } else {
            self
        }
    }

    fn convert_to_multi_transfer_esdt_call(self) -> Self {
        let payments = self.no_payments();

        let mut new_arg_buffer = ManagedArgBuffer::new();
        new_arg_buffer.push_arg(self.to);
        new_arg_buffer.push_arg(self.payments.len());

        for payment in self.payments.into_iter() {
            new_arg_buffer.push_arg(payment.token_identifier);
            new_arg_buffer.push_arg(payment.token_nonce);
            new_arg_buffer.push_arg(payment.amount);
        }
        if !self.endpoint_name.is_empty() {
            new_arg_buffer.push_arg(self.endpoint_name);
        }

        // multi transfer is sent to self, sender = receiver
        let recipient_addr = BlockchainWrapper::<SA>::new().get_sc_address();
        let zero = BigUint::zero();
        let endpoint_name = ManagedBuffer::from(ESDT_MULTI_TRANSFER_FUNC_NAME);

        ContractCall {
            _phantom: PhantomData,
            to: recipient_addr,
            egld_payment: zero,
            payments,
            explicit_gas_limit: self.explicit_gas_limit,
            endpoint_name,
            arg_buffer: new_arg_buffer.concat(self.arg_buffer),
            _return_type: PhantomData,
        }
    }

    pub fn resolve_gas_limit(&self) -> u64 {
        if self.explicit_gas_limit == UNSPECIFIED_GAS_LIMIT {
            SA::blockchain_api_impl().get_gas_left()
        } else {
            self.explicit_gas_limit
        }
    }

    pub fn async_call(mut self) -> AsyncCall<SA> {
        self = self.convert_to_esdt_transfer_call();
        AsyncCall {
            to: self.to,
            egld_payment: self.egld_payment,
            endpoint_name: self.endpoint_name,
            arg_buffer: self.arg_buffer,
            callback_call: None,
        }
    }

    #[cfg(feature = "promises")]
    pub fn async_call_promise(mut self) -> super::AsyncCallPromises<SA> {
        self = self.convert_to_esdt_transfer_call();
        super::AsyncCallPromises {
            to: self.to,
            egld_payment: self.egld_payment,
            endpoint_name: self.endpoint_name,
            arg_buffer: self.arg_buffer,
            explicit_gas_limit: self.explicit_gas_limit,
            extra_gas_for_callback: 0,
            callback_call: None,
        }
    }
}

impl<SA, OriginalResult> ContractCall<SA, OriginalResult>
where
    SA: CallTypeApi + 'static,
    OriginalResult: TopEncodeMulti,
{
    fn decode_result<RequestedResult>(
        raw_result: ManagedVec<SA, ManagedBuffer<SA>>,
    ) -> RequestedResult
    where
        RequestedResult: TopDecodeMulti,
    {
        let mut loader = ManagedResultArgLoader::new(raw_result);
        let arg_id = ArgId::from(&b"sync result"[..]);
        let h = ArgErrorHandler::<SA>::from(arg_id);
        let Ok(result) = RequestedResult::multi_decode_or_handle_err(&mut loader, h);
        result
    }

    /// Executes immediately, synchronously, and returns contract call result.
    /// Only works if the target contract is in the same shard.
    pub fn execute_on_dest_context<RequestedResult>(mut self) -> RequestedResult
    where
        RequestedResult: TopDecodeMulti,
    {
        self = self.convert_to_esdt_transfer_call();
        let raw_result = SendRawWrapper::<SA>::new().execute_on_dest_context_raw(
            self.resolve_gas_limit(),
            &self.to,
            &self.egld_payment,
            &self.endpoint_name,
            &self.arg_buffer,
        );

        SendRawWrapper::<SA>::new().clean_return_data();

        Self::decode_result(raw_result)
    }

    pub fn execute_on_dest_context_readonly<RequestedResult>(mut self) -> RequestedResult
    where
        RequestedResult: TopDecodeMulti,
    {
        self = self.convert_to_esdt_transfer_call();
        let raw_result = SendRawWrapper::<SA>::new().execute_on_dest_context_readonly_raw(
            self.resolve_gas_limit(),
            &self.to,
            &self.endpoint_name,
            &self.arg_buffer,
        );

        SendRawWrapper::<SA>::new().clean_return_data();

        Self::decode_result(raw_result)
    }

    pub fn execute_on_same_context<RequestedResult>(mut self) -> RequestedResult
    where
        RequestedResult: TopDecodeMulti,
    {
        self = self.convert_to_esdt_transfer_call();
        let raw_result = SendRawWrapper::<SA>::new().execute_on_same_context_raw(
            self.resolve_gas_limit(),
            &self.to,
            &self.egld_payment,
            &self.endpoint_name,
            &self.arg_buffer,
        );

        SendRawWrapper::<SA>::new().clean_return_data();

        Self::decode_result(raw_result)
    }
}

impl<SA, OriginalResult> ContractCall<SA, OriginalResult>
where
    SA: CallTypeApi + 'static,
{
    fn resolve_gas_limit_with_leftover(&self) -> u64 {
        if self.explicit_gas_limit == UNSPECIFIED_GAS_LIMIT {
            let mut gas_left = SA::blockchain_api_impl().get_gas_left();
            if gas_left > TRANSFER_EXECUTE_DEFAULT_LEFTOVER {
                gas_left -= TRANSFER_EXECUTE_DEFAULT_LEFTOVER;
            }
            gas_left
        } else {
            self.explicit_gas_limit
        }
    }

    /// Immediately launches a transfer-execute call.
    ///
    /// This is similar to an async call, but there is no callback
    /// and there can be more than one such call per transaction.
    pub fn transfer_execute(self) {
        match self.payments.len() {
            0 => self.no_payment_transfer_execute(),
            1 => self.single_transfer_execute(),
            _ => self.multi_transfer_execute(),
        }
    }

    fn no_payment_transfer_execute(&self) {
        let gas_limit = self.resolve_gas_limit_with_leftover();

        let _ = SendRawWrapper::<SA>::new().direct_egld_execute(
            &self.to,
            &self.egld_payment,
            gas_limit,
            &self.endpoint_name,
            &self.arg_buffer,
        );
    }

    fn single_transfer_execute(self) {
        let gas_limit = self.resolve_gas_limit_with_leftover();
        let payment = &self.payments.try_get(0).unwrap();

        if self.egld_payment > 0 {
            let _ = SendRawWrapper::<SA>::new().direct_egld_execute(
                &self.to,
                &self.egld_payment,
                gas_limit,
                &self.endpoint_name,
                &self.arg_buffer,
            );
        } else if payment.token_nonce == 0 {
            // fungible ESDT
            let _ = SendRawWrapper::<SA>::new().transfer_esdt_execute(
                &self.to,
                &payment.token_identifier,
                &payment.amount,
                gas_limit,
                &self.endpoint_name,
                &self.arg_buffer,
            );
        } else {
            // non-fungible/semi-fungible ESDT
            let _ = SendRawWrapper::<SA>::new().transfer_esdt_nft_execute(
                &self.to,
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
                gas_limit,
                &self.endpoint_name,
                &self.arg_buffer,
            );
        }
    }

    fn multi_transfer_execute(self) {
        let gas_limit = self.resolve_gas_limit_with_leftover();
        let _ = SendRawWrapper::<SA>::new().multi_esdt_transfer_execute(
            &self.to,
            &self.payments,
            gas_limit,
            &self.endpoint_name,
            &self.arg_buffer,
        );
    }
}
