// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            7
// Async Callback (empty):               1
// Total number of exported functions:   9

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    first_contract
    (
        init => init
        transferToSecondContractFull => transfer_to_second_contract_full
        transferToSecondContractHalf => transfer_to_second_contract_half
        transferToSecondContractRejected => transfer_to_second_contract_rejected
        transferToSecondContractRejectedWithTransferAndExecute => transfer_to_second_contract_rejected_with_transfer_and_execute
        transferToSecondContractFullWithTransferAndExecute => transfer_to_second_contract_full_with_transfer_and_execute
        getesdtTokenName => get_contract_esdt_token_identifier
        getSecondContractAddress => get_second_contract_address
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
