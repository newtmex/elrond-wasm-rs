// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            8
// Async Callback (empty):               1
// Total number of exported functions:  10

#![no_std]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    digital_cash
    (
        init => init
        fund => fund
        withdraw => withdraw
        claim => claim
        claim_fees => claim_fees
        deposit_fees => deposit_fees
        forward => forward
        amount => get_amount
        deposit => deposit
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
