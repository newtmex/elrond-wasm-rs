use core::marker::PhantomData;

use super::properties::*;
use hex_literal::hex;

use crate::{
    api::{CallTypeApi, SendApi},
    types::{
        BigUint, ContractCall, EsdtLocalRole, EsdtTokenType, ManagedAddress, ManagedBuffer,
        TokenIdentifier,
    },
};

/// Address of the system smart contract that manages ESDT.
/// Bech32: erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
pub const ESDT_SYSTEM_SC_ADDRESS_ARRAY: [u8; 32] =
    hex!("000000000000000000010000000000000000000000000000000000000002ffff");

const ISSUE_FUNGIBLE_ENDPOINT_NAME: &[u8] = b"issue";
const ISSUE_NON_FUNGIBLE_ENDPOINT_NAME: &[u8] = b"issueNonFungible";
const ISSUE_SEMI_FUNGIBLE_ENDPOINT_NAME: &[u8] = b"issueSemiFungible";
const REGISTER_META_ESDT_ENDPOINT_NAME: &[u8] = b"registerMetaESDT";
const ISSUE_AND_SET_ALL_ROLES_ENDPOINT_NAME: &[u8] = b"registerAndSetAllRoles";

/// Proxy for the ESDT system smart contract.
/// Unlike other contract proxies, this one has a fixed address,
/// so the proxy object doesn't really contain any data, it is more of a placeholder.
pub struct ESDTSystemSmartContractProxy<SA>
where
    SA: SendApi + 'static,
{
    _phantom: PhantomData<SA>,
}

impl<SA> ESDTSystemSmartContractProxy<SA>
where
    SA: SendApi + 'static,
{
    /// Constructor.
    /// TODO: consider moving this to a new Proxy contructor trait (bonus: better proxy constructor syntax).
    pub fn new_proxy_obj() -> Self {
        ESDTSystemSmartContractProxy {
            _phantom: PhantomData,
        }
    }
}

impl<SA> ESDTSystemSmartContractProxy<SA>
where
    SA: CallTypeApi + 'static,
{
    /// Produces a contract call to the ESDT system SC,
    /// which causes it to issue a new fungible ESDT token.
    pub fn issue_fungible(
        self,
        issue_cost: BigUint<SA>,
        token_display_name: &ManagedBuffer<SA>,
        token_ticker: &ManagedBuffer<SA>,
        initial_supply: &BigUint<SA>,
        properties: FungibleTokenProperties,
    ) -> ContractCall<SA, ()> {
        self.issue(
            issue_cost,
            EsdtTokenType::Fungible,
            token_display_name,
            token_ticker,
            initial_supply,
            TokenProperties {
                num_decimals: properties.num_decimals,
                can_freeze: properties.can_freeze,
                can_wipe: properties.can_wipe,
                can_pause: properties.can_pause,
                can_transfer_create_role: false,
                can_mint: properties.can_mint,
                can_burn: properties.can_burn,
                can_change_owner: properties.can_change_owner,
                can_upgrade: properties.can_upgrade,
                can_add_special_roles: properties.can_add_special_roles,
            },
        )
    }

    /// Produces a contract call to the ESDT system SC,
    /// which causes it to issue a new non-fungible ESDT token.
    pub fn issue_non_fungible(
        self,
        issue_cost: BigUint<SA>,
        token_display_name: &ManagedBuffer<SA>,
        token_ticker: &ManagedBuffer<SA>,
        properties: NonFungibleTokenProperties,
    ) -> ContractCall<SA, ()> {
        let zero = BigUint::zero();
        self.issue(
            issue_cost,
            EsdtTokenType::NonFungible,
            token_display_name,
            token_ticker,
            &zero,
            TokenProperties {
                num_decimals: 0,
                can_freeze: properties.can_freeze,
                can_wipe: properties.can_wipe,
                can_pause: properties.can_pause,
                can_transfer_create_role: properties.can_transfer_create_role,
                can_mint: false,
                can_burn: false,
                can_change_owner: properties.can_change_owner,
                can_upgrade: properties.can_upgrade,
                can_add_special_roles: properties.can_add_special_roles,
            },
        )
    }

    /// Produces a contract call to the ESDT system SC,
    /// which causes it to issue a new semi-fungible ESDT token.
    pub fn issue_semi_fungible(
        self,
        issue_cost: BigUint<SA>,
        token_display_name: &ManagedBuffer<SA>,
        token_ticker: &ManagedBuffer<SA>,
        properties: SemiFungibleTokenProperties,
    ) -> ContractCall<SA, ()> {
        let zero = BigUint::zero();
        self.issue(
            issue_cost,
            EsdtTokenType::SemiFungible,
            token_display_name,
            token_ticker,
            &zero,
            TokenProperties {
                num_decimals: 0,
                can_freeze: properties.can_freeze,
                can_wipe: properties.can_wipe,
                can_pause: properties.can_pause,
                can_transfer_create_role: properties.can_transfer_create_role,
                can_mint: false,
                can_burn: false,
                can_change_owner: properties.can_change_owner,
                can_upgrade: properties.can_upgrade,
                can_add_special_roles: properties.can_add_special_roles,
            },
        )
    }

    /// Produces a contract call to the ESDT system SC,
    /// which causes it to register a new Meta ESDT token.
    pub fn register_meta_esdt(
        self,
        issue_cost: BigUint<SA>,
        token_display_name: &ManagedBuffer<SA>,
        token_ticker: &ManagedBuffer<SA>,
        properties: MetaTokenProperties,
    ) -> ContractCall<SA, ()> {
        let zero = BigUint::zero();
        self.issue(
            issue_cost,
            EsdtTokenType::Meta,
            token_display_name,
            token_ticker,
            &zero,
            TokenProperties {
                num_decimals: properties.num_decimals,
                can_freeze: properties.can_freeze,
                can_wipe: properties.can_wipe,
                can_pause: properties.can_pause,
                can_transfer_create_role: properties.can_transfer_create_role,
                can_mint: false,
                can_burn: false,
                can_change_owner: properties.can_change_owner,
                can_upgrade: properties.can_upgrade,
                can_add_special_roles: properties.can_add_special_roles,
            },
        )
    }

    pub fn issue_and_set_all_roles(
        self,
        issue_cost: BigUint<SA>,
        token_display_name: ManagedBuffer<SA>,
        token_ticker: ManagedBuffer<SA>,
        token_type: EsdtTokenType,
        num_decimals: usize,
    ) -> ContractCall<SA, ()> {
        let esdt_system_sc_address = self.esdt_system_sc_address();

        let mut contract_call = ContractCall::new(
            esdt_system_sc_address,
            ManagedBuffer::new_from_bytes(ISSUE_AND_SET_ALL_ROLES_ENDPOINT_NAME),
        )
        .with_egld_transfer(issue_cost);

        contract_call.push_endpoint_arg(&token_display_name);
        contract_call.push_endpoint_arg(&token_ticker);

        let token_type_name = match token_type {
            EsdtTokenType::Fungible => &b"FNG"[..],
            EsdtTokenType::NonFungible => &b"NFT"[..],
            EsdtTokenType::SemiFungible => &b"SFT"[..],
            EsdtTokenType::Meta => &b"META"[..],
            EsdtTokenType::Invalid => &[],
        };
        contract_call.push_endpoint_arg(&token_type_name);
        contract_call.push_endpoint_arg(&num_decimals);

        contract_call
    }

    /// Deduplicates code from all the possible issue functions
    fn issue(
        self,
        issue_cost: BigUint<SA>,
        token_type: EsdtTokenType,
        token_display_name: &ManagedBuffer<SA>,
        token_ticker: &ManagedBuffer<SA>,
        initial_supply: &BigUint<SA>,
        properties: TokenProperties,
    ) -> ContractCall<SA, ()> {
        let esdt_system_sc_address = self.esdt_system_sc_address();

        let endpoint_name = match token_type {
            EsdtTokenType::Fungible => ISSUE_FUNGIBLE_ENDPOINT_NAME,
            EsdtTokenType::NonFungible => ISSUE_NON_FUNGIBLE_ENDPOINT_NAME,
            EsdtTokenType::SemiFungible => ISSUE_SEMI_FUNGIBLE_ENDPOINT_NAME,
            EsdtTokenType::Meta => REGISTER_META_ESDT_ENDPOINT_NAME,
            EsdtTokenType::Invalid => &[],
        };

        let mut contract_call = ContractCall::new(
            esdt_system_sc_address,
            ManagedBuffer::new_from_bytes(endpoint_name),
        )
        .with_egld_transfer(issue_cost);

        contract_call.push_endpoint_arg(token_display_name);
        contract_call.push_endpoint_arg(token_ticker);

        if token_type == EsdtTokenType::Fungible {
            contract_call.push_endpoint_arg(initial_supply);
            contract_call.push_endpoint_arg(&properties.num_decimals);
        } else if token_type == EsdtTokenType::Meta {
            contract_call.push_endpoint_arg(&properties.num_decimals);
        }

        let mut token_prop_args = TokenPropertyArguments {
            can_freeze: Some(properties.can_freeze),
            can_wipe: Some(properties.can_wipe),
            can_pause: Some(properties.can_pause),
            can_change_owner: Some(properties.can_change_owner),
            can_upgrade: Some(properties.can_upgrade),
            can_add_special_roles: Some(properties.can_add_special_roles),
            ..TokenPropertyArguments::default()
        };

        if token_type == EsdtTokenType::Fungible {
            token_prop_args.can_mint = Some(properties.can_mint);
            token_prop_args.can_burn = Some(properties.can_burn);
        } else {
            token_prop_args.can_transfer_create_role = Some(properties.can_transfer_create_role);
        }

        append_token_property_arguments(&mut contract_call, &token_prop_args);

        contract_call
    }

    /// Produces a contract call to the ESDT system SC,
    /// which causes it to mint more fungible ESDT tokens.
    /// It will fail if the SC is not the owner of the token.
    pub fn mint(
        self,
        token_identifier: &TokenIdentifier<SA>,
        amount: &BigUint<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"mint");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(amount);

        contract_call
    }

    /// Produces a contract call to the ESDT system SC,
    /// which causes it to burn fungible ESDT tokens owned by the SC.
    pub fn burn(
        self,
        token_identifier: &TokenIdentifier<SA>,
        amount: &BigUint<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"ESDTBurn");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(amount);

        contract_call
    }

    /// The manager of an ESDT token may choose to suspend all transactions of the token,
    /// except minting, freezing/unfreezing and wiping.
    pub fn pause(self, token_identifier: &TokenIdentifier<SA>) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"pause");

        contract_call.push_endpoint_arg(token_identifier);

        contract_call
    }

    /// The reverse operation of `pause`.
    pub fn unpause(self, token_identifier: &TokenIdentifier<SA>) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"unPause");

        contract_call.push_endpoint_arg(token_identifier);

        contract_call
    }

    /// The manager of an ESDT token may freeze the tokens held by a specific account.
    /// As a consequence, no tokens may be transferred to or from the frozen account.
    /// Freezing and unfreezing the tokens of an account are operations designed to help token managers to comply with regulations.
    pub fn freeze(
        self,
        token_identifier: &TokenIdentifier<SA>,
        address: &ManagedAddress<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"freeze");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(address);

        contract_call
    }

    /// The reverse operation of `freeze`, unfreezing, will allow further transfers to and from the account.
    pub fn unfreeze(
        self,
        token_identifier: &TokenIdentifier<SA>,
        address: &ManagedAddress<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"unFreeze");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(address);

        contract_call
    }

    /// The manager of an ESDT token may wipe out all the tokens held by a frozen account.
    /// This operation is similar to burning the tokens, but the account must have been frozen beforehand,
    /// and it must be done by the token manager.
    /// Wiping the tokens of an account is an operation designed to help token managers to comply with regulations.
    pub fn wipe(
        self,
        token_identifier: &TokenIdentifier<SA>,
        address: &ManagedAddress<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"wipe");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(address);

        contract_call
    }

    /// This function converts an SFT to a metaESDT by adding decimals to its structure in the metachain ESDT System SC.
    /// This function as almost all in case of ESDT can be called only by the owner.
    pub fn change_sft_to_meta_esdt(
        self,
        token_identifier: &TokenIdentifier<SA>,
        num_decimals: usize,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"changeSFTToMetaESDT");

        contract_call.push_endpoint_arg(&token_identifier);
        contract_call.push_endpoint_arg(&num_decimals);

        contract_call
    }

    /// This function can be called only if canSetSpecialRoles was set to true.
    /// The metachain system SC will evaluate the arguments and call “ESDTSetRole@tokenId@listOfRoles” for the given address.
    /// This will be actually a cross shard call.
    /// This function as almost all in case of ESDT can be called only by the owner.
    pub fn set_special_roles<RoleIter: Iterator<Item = EsdtLocalRole>>(
        self,
        address: &ManagedAddress<SA>,
        token_identifier: &TokenIdentifier<SA>,
        roles_iter: RoleIter,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"setSpecialRole");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(address);
        for role in roles_iter {
            if role != EsdtLocalRole::None {
                contract_call.push_argument_raw_bytes(role.as_role_name());
            }
        }

        contract_call
    }

    /// This function can be called only if canSetSpecialRoles was set to true.
    /// The metachain system SC will evaluate the arguments and call “ESDTUnsetRole@tokenId@listOfRoles” for the given address.
    /// This will be actually a cross shard call.
    /// This function as almost all in case of ESDT can be called only by the owner.
    pub fn unset_special_roles<RoleIter: Iterator<Item = EsdtLocalRole>>(
        self,
        address: &ManagedAddress<SA>,
        token_identifier: &TokenIdentifier<SA>,
        roles_iter: RoleIter,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"unSetSpecialRole");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(address);
        for role in roles_iter {
            if role != EsdtLocalRole::None {
                contract_call.push_argument_raw_bytes(role.as_role_name());
            }
        }

        contract_call
    }

    pub fn transfer_ownership(
        self,
        token_identifier: &TokenIdentifier<SA>,
        new_owner: &ManagedAddress<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"transferOwnership");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(new_owner);

        contract_call
    }

    pub fn transfer_nft_create_role(
        self,
        token_identifier: &TokenIdentifier<SA>,
        old_creator: &ManagedAddress<SA>,
        new_creator: &ManagedAddress<SA>,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"transferNFTCreateRole");

        contract_call.push_endpoint_arg(token_identifier);
        contract_call.push_endpoint_arg(old_creator);
        contract_call.push_endpoint_arg(new_creator);

        contract_call
    }

    pub fn control_changes(
        self,
        token_identifier: &TokenIdentifier<SA>,
        property_arguments: &TokenPropertyArguments,
    ) -> ContractCall<SA, ()> {
        let mut contract_call = self.esdt_system_sc_call_no_args(b"controlChanges");
        contract_call.push_endpoint_arg(token_identifier);
        append_token_property_arguments(&mut contract_call, property_arguments);
        contract_call
    }

    pub fn esdt_system_sc_address(&self) -> ManagedAddress<SA> {
        ManagedAddress::new_from_bytes(&ESDT_SYSTEM_SC_ADDRESS_ARRAY)
    }

    fn esdt_system_sc_call_no_args(self, endpoint_name: &[u8]) -> ContractCall<SA, ()> {
        let esdt_system_sc_address = self.esdt_system_sc_address();
        ContractCall::new(
            esdt_system_sc_address,
            ManagedBuffer::new_from_bytes(endpoint_name),
        )
    }
}

const TRUE_BYTES: &[u8] = b"true";
const FALSE_BYTES: &[u8] = b"false";

fn bool_name_bytes(b: bool) -> &'static [u8] {
    if b {
        TRUE_BYTES
    } else {
        FALSE_BYTES
    }
}

fn set_token_property<SA, R>(contract_call: &mut ContractCall<SA, R>, name: &[u8], value: bool)
where
    SA: CallTypeApi + 'static,
{
    contract_call.push_argument_raw_bytes(name);
    contract_call.push_argument_raw_bytes(bool_name_bytes(value));
}

fn append_token_property_arguments<SA, R>(
    contract_call: &mut ContractCall<SA, R>,
    token_prop_args: &TokenPropertyArguments,
) where
    SA: CallTypeApi + 'static,
{
    if let Some(can_freeze) = token_prop_args.can_freeze {
        set_token_property(contract_call, &b"canFreeze"[..], can_freeze);
    }

    if let Some(can_wipe) = token_prop_args.can_wipe {
        set_token_property(contract_call, &b"canWipe"[..], can_wipe);
    }

    if let Some(can_pause) = token_prop_args.can_pause {
        set_token_property(contract_call, &b"canPause"[..], can_pause);
    }

    if let Some(can_transfer_create_role) = token_prop_args.can_transfer_create_role {
        set_token_property(
            contract_call,
            &b"canTransferNFTCreateRole"[..],
            can_transfer_create_role,
        );
    }

    if let Some(can_mint) = token_prop_args.can_mint {
        set_token_property(contract_call, &b"canMint"[..], can_mint);
    }

    if let Some(can_burn) = token_prop_args.can_burn {
        set_token_property(contract_call, &b"canBurn"[..], can_burn);
    }

    if let Some(can_change_owner) = token_prop_args.can_change_owner {
        set_token_property(contract_call, &b"canChangeOwner"[..], can_change_owner);
    }

    if let Some(can_upgrade) = token_prop_args.can_upgrade {
        set_token_property(contract_call, &b"canUpgrade"[..], can_upgrade);
    }

    if let Some(can_add_special_roles) = token_prop_args.can_add_special_roles {
        set_token_property(
            contract_call,
            &b"canAddSpecialRoles"[..],
            can_add_special_roles,
        );
    }
}
