// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            7
// Async Callback (empty):               1
// Total number of exported functions:   9

#![no_std]

mx_sc_wasm_adapter::wasm_endpoints! {
    lottery_esdt
    (
        start
        createLotteryPool
        buy_ticket
        determine_winner
        status
        getLotteryInfo
        getLotteryWhitelist
    )
}

mx_sc_wasm_adapter::wasm_empty_callback! {}
