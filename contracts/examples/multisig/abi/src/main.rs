use multisig::*;
use elrond_wasm_debug::*;

fn main() {
	let contract = MultisigImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
