# The MultiversX Rust Tool Set

This repository contains a wide variety of tools, aimed primarily at smart contract developers.

The repo contains:
- The most complete smart contract framework on MultiversX:
    - The base framework;
    - A complete build system, which relies on the smart contract code directly;
    - A powerful debugger, based on a partial implementation of the MultiversX VM, in Rust.
    - A framework for writing both black-box and white-box tests. They rely on the standard MultiversX blockchain scenario format.
    - The official data serializer and deserializer for smart contract data. Can be used both on- and off-chain.
- A large collection of smart contract examples and feature tests, together with some of the core smart contracts used on the blockchain (e.g. the wrapped egld swap, multisig, etc.).
- A framework for interacting with the blockchain, based on the smart contract logic, especially suitable for developers.
- A code snippet generator.

# Documentation

Most documentation can be found at https://docs.multiversx.com/developers/overview/

# Getting started

The crowdfunding tutorial is a great place to start: https://docs.multiversx.com/developers/tutorials/crowdfunding-p1/

# IDE

The framework is designed to be easiest to use with the Elrond IDE VSCode extension: https://marketplace.visualstudio.com/items?itemName=Elrond.vscode-elrond-ide

# Building contracts

A comprehensive build guide can be found here: https://docs.multiversx.com/developers/developer-reference/smart-contract-build-reference/

# Debugging contracts

The debugger guide: https://docs.multiversx.com/developers/developer-reference/rust-smart-contract-debugging/
