#![no_std]
#![allow(unused_attributes)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod deposit_info;

use deposit_info::{DepositInfo, FundType};

pub const SECONDS_PER_ROUND: u64 = 6;
pub use multiversx_sc::api::{ED25519_KEY_BYTE_LEN, ED25519_SIGNATURE_BYTE_LEN};

#[multiversx_sc::contract]
pub trait DigitalCash {
    #[init]
    fn init(&self) {}

    //endpoints

    #[endpoint]
    #[payable("*")]
    fn fund(&self, address: ManagedAddress, valability: u64) {
        let payment = self.call_value().egld_or_single_esdt();
        require!(
            payment.amount > BigUint::zero(),
            "amount must be greater than 0"
        );
        let fund_type = FundType {
            token: payment.token_identifier.clone(),
            nonce: payment.token_nonce.clone(),
        };

        let mut deposit = DepositInfo {
            payment,
            expiration_round: self.get_expiration_round(valability),
        };

        self.deposit(&address).entry(fund_type).and_modify(|fund| {
            deposit.payment.amount += fund.payment.amount.clone();
            deposit.expiration_round = deposit.expiration_round.max(fund.expiration_round);
        });
    }

    #[endpoint]
    fn withdraw(&self, address: ManagedAddress) {
        require!(!self.deposit(&address).is_empty(), "non-existent key");

        for (key, deposit) in self.deposit(&address).iter() {
            if deposit.expiration_round < self.blockchain().get_block_round() {
                continue;
            }
            self.send().direct(
                &address,
                &deposit.payment.token_identifier,
                deposit.payment.token_nonce,
                &deposit.payment.amount,
            );
            self.deposit(&address).remove(&key);
        }
    }

    #[endpoint]
    fn claim(
        &self,
        address: ManagedAddress,
        signature: ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>,
    ) {
        require!(!self.deposit(&address).is_empty(), "non-existent key");

        let caller_address = self.blockchain().get_caller();

        let addr = address.as_managed_byte_array();

        let message = caller_address.as_managed_buffer();

        for (key, deposit) in self.deposit(&address).iter() {
            if deposit.expiration_round >= self.blockchain().get_block_round() {
                continue;
            }
            require!(
                self.crypto()
                    .verify_ed25519_legacy_managed::<32>(addr, message, &signature),
                "invalid signature"
            );

            self.send().direct(
                &caller_address,
                &deposit.payment.token_identifier,
                deposit.payment.token_nonce,
                &deposit.payment.amount,
            );
            self.deposit(&address).remove(&key);
        }
    }

    //views

    #[view(amount)]
    fn get_amount(
        &self,
        address: ManagedAddress,
        token: EgldOrEsdtTokenIdentifier,
        nonce: u64,
    ) -> BigUint {
        require!(!self.deposit(&address).is_empty(), "non-existent key");

        let data = self.deposit(&address).get(&FundType { token, nonce });
        let mut amount = BigUint::zero();
        if let Some(fund) = data {
            amount = fund.payment.amount;
        } else {
            require!(!self.deposit(&address).is_empty(), "non-existent key");
        }
        amount
    }

    //private functions

    fn get_expiration_round(&self, valability: u64) -> u64 {
        let valability_rounds = valability / SECONDS_PER_ROUND;
        self.blockchain().get_block_round() + valability_rounds
    }

    //storage

    #[view]
    #[storage_mapper("deposit")]
    fn deposit(
        &self,
        donor: &ManagedAddress,
    ) -> MapMapper<FundType<Self::Api>, DepositInfo<Self::Api>>;
}
