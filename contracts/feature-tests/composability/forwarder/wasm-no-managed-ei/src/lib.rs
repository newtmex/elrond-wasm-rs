////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    forwarder
    (
        init
        callBack
        buy_nft
        callback_data
        callback_data_at_index
        changeOwnerAddress
        clear_callback_data
        create_and_send
        deploy_contract
        deploy_two_contracts
        deploy_vault_from_source
        echo_arguments_sync
        echo_arguments_sync_range
        echo_arguments_sync_twice
        forward_async_accept_funds
        forward_async_accept_funds_half_payment
        forward_async_accept_funds_with_fees
        forward_async_retrieve_funds
        forward_sync_accept_funds
        forward_sync_accept_funds_multi_transfer
        forward_sync_accept_funds_then_read
        forward_sync_accept_funds_with_fees
        forward_sync_retrieve_funds
        forward_transf_exec_accept_funds
        forward_transf_exec_accept_funds_multi_transfer
        forward_transf_exec_accept_funds_return_values
        forward_transf_exec_accept_funds_twice
        forward_transf_execu_accept_funds_with_fees
        getCurrentNftNonce
        getFungibleEsdtBalance
        get_esdt_local_roles
        check_token_has_roles
        get_nft_balance
        issue_fungible_token
        lastErrorMessage
        lastIssuedToken
        local_burn
        local_mint
        multi_transfer_via_async
        nft_add_quantity
        nft_burn
        nft_create
        nft_create_on_caller_behalf
        nft_decode_complex_attributes
        nft_issue
        send_egld
        send_esdt
        send_esdt_direct_multi_transfer
        send_esdt_twice
        send_esdt_with_fees
        send_funds_twice
        setLocalRoles
        sft_issue
        transfer_nft_and_execute
        transfer_nft_via_async_call
        unsetLocalRoles
        upgradeVault
        upgrade_vault_from_source
    )
}
