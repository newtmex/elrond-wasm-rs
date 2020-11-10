#!/bin/sh

# helper script for checking that all contracts and debug projects compile

### EXAMPLES ###

export RUSTFLAGS=${RUSTFLAGS-'-C link-arg=-s'}

cd contracts/benchmarks/str-repeat/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/str_repeat_wasm.wasm output/str-repeat.wasm
cd ../../..

cd contracts/examples/adder/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/adder_wasm.wasm output/adder.wasm
cd ../../..

cd contracts/examples/crypto-bubbles/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/crypto_bubbles_wasm.wasm output/crypto-bubbles.wasm
cd ../../..

cd contracts/examples/factorial/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/factorial_wasm.wasm output/factorial.wasm
cd ../../..

cd contracts/examples/simple-erc20/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/simple_erc20_wasm.wasm output/simple-erc20.wasm
cd ../../..


### TEST CONTRACTS ###

cd contracts/feature-tests/basic-features/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/basic_features_wasm.wasm output/features.wasm
cd ../../..

cd contracts/feature-tests/async/async-alice/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/async_alice_wasm.wasm output/alice.wasm
cd ../../../..

cd contracts/feature-tests/async/async-bob/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/async_bob_wasm.wasm output/bob.wasm
cd ../../../..

cd contracts/feature-tests/use-module/wasm
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/use_module_wasm.wasm output/use_module.wasm
cd ../../..
