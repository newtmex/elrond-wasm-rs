#![no_std]

elrond_wasm::imports!();

/// Test contract for investigating async calls.
#[elrond_wasm_derive::contract(ForwarderRawImpl)]
pub trait ForwarderRaw {
	#[init]
	fn init(&self) {}

	#[endpoint]
	#[payable("*")]
	fn forward_payment(
		&self,
		to: Address,
		#[payment_token] token: TokenIdentifier,
		#[payment] payment: BigUint,
	) -> SendToken<BigUint> {
		SendToken {
			to,
			token,
			amount: payment,
			data: BoxedBytes::empty(),
		}
	}

	#[endpoint]
	#[payable("*")]
	fn forward_async_call(
		&self,
		to: Address,
		#[payment_token] token: TokenIdentifier,
		#[payment] payment: BigUint,
		endpoint_name: BoxedBytes,
		#[var_args] args: VarArgs<BoxedBytes>,
	) -> AsyncCall<BigUint> {
		let mut contract_call = ContractCall::new(to, token, payment, endpoint_name);
		for arg in args.into_vec() {
			contract_call.push_argument_raw_bytes(arg.as_slice());
		}
		contract_call.async_call()
	}

	#[endpoint]
	#[payable("*")]
	fn forward_async_call_half_payment(
		&self,
		to: Address,
		#[payment_token] token: TokenIdentifier,
		#[payment] payment: BigUint,
		endpoint_name: BoxedBytes,
		#[var_args] args: VarArgs<BoxedBytes>,
	) -> AsyncCall<BigUint> {
		let half_payment = payment / 2u32.into();
		self.forward_async_call(to, token, half_payment, endpoint_name, args)
	}

	#[view]
	#[storage_mapper("callback_data")]
	fn callback_data(
		&self,
	) -> VecMapper<Self::Storage, (TokenIdentifier, BigUint, Vec<BoxedBytes>)>;

	#[view]
	fn callback_data_at_index(
		&self,
		index: usize,
	) -> MultiResult3<TokenIdentifier, BigUint, MultiResultVec<BoxedBytes>> {
		let (token, payment, args) = self.callback_data().get(index);
		(token, payment, args.into()).into()
	}

	#[endpoint]
	fn clear_callback_info(&self) {
		self.callback_data().clear();
	}

	#[callback_raw]
	fn callback_raw(
		&self,
		#[payment_token] token: TokenIdentifier,
		#[payment] payment: BigUint,
		#[var_args] args: VarArgs<BoxedBytes>,
	) {
		self.callback_data()
			.push(&(token, payment, args.into_vec()));
	}
}
