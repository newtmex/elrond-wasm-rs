// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            8
// Async Callback:                       1
// Total number of exported functions:  10

#![no_std]

mx_sc_wasm_adapter::wasm_endpoints! {
    crowdfunding_erc20
    (
        fund
        status
        claim
        get_target
        get_deadline
        get_deposit
        get_erc20_contract_address
        get_total_balance
        callBack
    )
}
