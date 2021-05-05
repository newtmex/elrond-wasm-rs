use super::{Method, Supertrait};

/// Models a contract or module trait.
pub struct ContractTrait {
	pub docs: Vec<String>,
	pub trait_name: proc_macro2::Ident,
	pub contract_impl_name: syn::Path,
	pub supertraits: Vec<Supertrait>,

	/// It is possible to automatically implement a contract module for all contracts that use it indirectly.
	/// The drawback is that the developer make sure multiple inheritance does not happen.
	/// This feature is currently disabled.
	pub auto_inheritance_modules: Vec<Supertrait>,

	pub methods: Vec<Method>,
}
