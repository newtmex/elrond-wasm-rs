// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            8
// Async Callback (empty):               1
// Total number of exported functions:  10

#![no_std]

mx_sc_wasm_adapter::wasm_endpoints! {
    linked_list_repeat
    (
        add
        count
        remove
        bench
        add_struct
        count_struct
        remove_struct
        bench_struct
    )
}

mx_sc_wasm_adapter::wasm_empty_callback! {}
