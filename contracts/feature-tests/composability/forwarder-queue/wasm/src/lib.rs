// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           10
// Async Callback (empty):               1
// Total number of exported functions:  12

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    forwarder_queue
    (
        init => init
        queued_calls => queued_calls
        add_queued_call_sync => add_queued_call_sync
        add_queued_call_legacy_async => add_queued_call_legacy_async
        add_queued_call_transfer_execute => add_queued_call_transfer_execute
        add_queued_call_transfer_esdt => add_queued_call_transfer_esdt
        add_queued_call_promise => add_queued_call_promise
        add_queued_call => add_queued_call
        forward_queued_calls => forward_queued_calls
        callback_count => callback_count
        callback_payments => callback_payments
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
