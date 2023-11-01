#![no_std]
#![allow(clippy::type_complexity)]

mod call_promise_direct;
mod call_promises;
mod call_sync_bt;

multiversx_sc::imports!();

/// Test contract for investigating the new async call framework.
#[multiversx_sc::contract]
pub trait PromisesFeatures:
    call_promises::CallPromisesModule
    + call_promise_direct::CallPromisesDirectModule
    + call_sync_bt::BackTransfersFeatureModule
{
    #[init]
    fn init(&self) {}
}
