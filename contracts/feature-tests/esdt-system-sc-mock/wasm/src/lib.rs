// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            6
// Async Callback (empty):               1
// Total number of exported functions:   8

#![no_std]

mx_sc_wasm_adapter::wasm_endpoints! {
    esdt_system_sc_mock
    (
        issue
        issueNonFungible
        issueSemiFungible
        registerMetaESDT
        setSpecialRole
        registerAndSetAllRoles
    )
}

mx_sc_wasm_adapter::wasm_empty_callback! {}
