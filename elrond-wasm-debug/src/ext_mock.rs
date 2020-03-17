

use elrond_wasm::{H256, Address, StorageKey};

use crate::big_int_mock::*;
use crate::big_uint_mock::*;

use elrond_wasm::ContractHookApi;
use elrond_wasm::CallableContract;
use elrond_wasm::BigUintApi;

use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use alloc::boxed::Box;
use alloc::vec::Vec;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use core::cell::RefCell;
use alloc::rc::Rc;

use sha3::{Sha3_256, Keccak256, Digest};

const ADDRESS_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32;
const TOPIC_LENGTH: usize = 32;

fn address_hex(address: &H256) -> alloc::string::String {
    alloc::format!("0x{}", hex::encode(address.as_bytes()))
}

pub struct AccountData {
    pub address: Address,
    pub nonce: u64,
    pub balance: BigInt,
    pub storage: HashMap<StorageKey, Vec<u8>>,
    pub contract: Option<Box<dyn CallableContract>>,
}

impl fmt::Display for AccountData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut storage_buf = String::new();
        let mut keys: Vec<StorageKey> = self.storage.iter().map(|(k, _)| k.clone()).collect();
        keys.sort_by(|k1, k2| k1.as_bytes().cmp(k2.as_bytes()));
        for key in &keys {
            let value = self.storage.get(&key).unwrap();
            write!(&mut storage_buf, "\n\t\t{} -> 0x{}", address_hex(key), hex::encode(value.as_slice())).unwrap();
        }
        
        write!(f, "AccountData {{ nonce: {}, balance: {}, storage: [{} ] }}",
            self.nonce, 
            self.balance,
            storage_buf)
    }
}

pub struct TxData {
    func_name: &'static str,
    new_contract: Option<Box<dyn CallableContract>>,
    args: Vec<Vec<u8>>,
    call_value: BigInt,
    from: Address,
    to: Address,
}

impl fmt::Display for TxData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxData {{ func: {}, args: {:?}, call_value: {}, from: 0x{}, to: 0x{}\n}}", 
            self.func_name, 
            self.args, 
            self.call_value,
            address_hex(&self.from), 
            address_hex(&self.to))
    }
}

impl TxData {
    pub fn new_create(new_contract: Box<dyn CallableContract>, from: Address, to: Address) -> Self {
        TxData{
            func_name: "init",
            new_contract: Some(new_contract),
            args: Vec::new(),
            call_value: 0.into(),
            from: from,
            to: to,
        }
    }

    pub fn new_call(func_name: &'static str, from: Address, to: Address) -> Self {
        TxData{
            func_name: func_name,
            new_contract: None,
            args: Vec::new(),
            call_value: 0.into(),
            from: from,
            to: to,
        }
    }

    pub fn add_arg(&mut self, arg: Vec<u8>) {
        self.args.push(arg);
    }
}

#[derive(Clone)]
pub struct TxResult {
    pub result_status: i32,
    pub result_values: Vec<Vec<u8>>,
}

impl fmt::Display for TxResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let results_hex: Vec<String> = self.result_values.iter().map(|r| format!("0x{}", hex::encode(r))).collect();
        write!(f, "TxResult {{\n\tresult_status: {},\n\tresult_values:{:?}\n}}", self.result_status, results_hex)
    }
}

impl TxResult {
    pub fn empty() -> TxResult {
        TxResult {
            result_status: 0,
            result_values: Vec::new(),
        }
    }
    pub fn print(&self) {
        print!("{}\n", self);
    }
}

pub struct ArwenMockState {
    current_tx: Option<TxData>,
    current_result: TxResult,
    accounts: HashMap<Address, AccountData>,
}

pub struct ArwenMockRef {
    state_ref: Rc<RefCell<ArwenMockState>>
}

impl Clone for ArwenMockRef {
    fn clone(&self) -> Self {
        ArwenMockRef{ state_ref: Rc::clone(&self.state_ref) }
    }
}

impl ArwenMockState {
    pub fn new() -> ArwenMockRef {
        let state = ArwenMockState{
            current_tx: None,
            current_result: TxResult::empty(),
            accounts: HashMap::new(),
        };
        let state_ref = Rc::new(RefCell::new(state));
        ArwenMockRef{ state_ref: state_ref }
    }

    fn create_account_if_necessary(&mut self, tx: &mut TxData) {
        if let Some(ref tx_contract) = tx.new_contract {
            if self.accounts.contains_key(&tx.to) {
                panic!("Account already exists");
            }
            self.accounts.insert(tx.to.clone(), AccountData{
                address: tx.to.clone(),
                nonce: 0,
                balance: 0.into(),
                storage: HashMap::new(),
                contract: Some(tx_contract.clone_contract()),
            });
        }
    }

    pub fn set_result_status(&mut self, status: i32) {
        self.current_result.result_status = status;
    }
    
    pub fn add_result(&mut self, result: Vec<u8>) {
        self.current_result.result_values.push(result);
    }
    
    fn clear_result(&mut self) {
        self.current_result = TxResult::empty();
    }
    
    fn get_result(&self) -> TxResult {
        self.current_result.clone()
    }
}

impl ArwenMockRef {
    fn get_contract(&self) -> Box<dyn CallableContract> {
        let state = self.state_ref.borrow();
        let tx_ref = &state.current_tx.as_ref().unwrap();
        match state.accounts.get(&tx_ref.to) {
            None => panic!("Account not found"),
            Some(ref account) => {
                match account.contract {
                    None => panic!("Recipient account is not a smart contract"),
                    Some(ref acct_contract) => acct_contract.clone_contract(),
                }
            }
        }
    }

    pub fn execute_tx(&self, mut tx: TxData) -> TxResult {
        {
            let mut state = self.state_ref.borrow_mut();
            state.create_account_if_necessary(&mut tx);    
            state.current_tx = Some(tx);
            state.clear_result();
        }
        
        let func_name = {
            let state = self.state_ref.borrow();
            let tx_ref = &state.current_tx.as_ref().unwrap();
            &tx_ref.func_name.clone()
        };
        
        let contract = self.get_contract();

        // contract call
        // important: state cannot be borrowed at this point
        contract.call(&func_name);
        
        let state = self.state_ref.borrow();
        state.get_result()
    }

    pub fn add_account(&self, acct: AccountData) {
        let mut state = self.state_ref.borrow_mut();
        state.accounts.insert(acct.address.clone(), acct);
    }

    pub fn print_accounts(&self) {
        let state = self.state_ref.borrow();
        let mut accounts_buf = String::new();
        for (address, account) in &state.accounts {
            write!(&mut accounts_buf, "\n\t{} -> {}", address_hex(address), account).unwrap();
        }
        print!("Accounts: {}\n", &accounts_buf);
    }
}

impl elrond_wasm::ContractHookApi<RustBigInt, RustBigUint> for ArwenMockRef {
    fn get_owner(&self) -> Address {
        let state = self.state_ref.borrow();
        match &state.current_tx {
            None => panic!("Tx not initialized!"),
            Some(tx) => tx.to.clone(),
        }
    }

    fn get_caller(&self) -> Address {
        let state = self.state_ref.borrow();
        match &state.current_tx {
            None => panic!("Tx not initialized!"),
            Some(tx) => tx.from.clone(),
        }
    }

    fn get_balance(&self, _address: &Address) -> RustBigUint {
        panic!("get_balance not yet implemented")
    }

    fn storage_store(&self, key: &StorageKey, value: &Vec<u8>) {
        let sc_address = self.get_owner();
        let mut state = self.state_ref.borrow_mut();
        match state.accounts.get_mut(&sc_address) {
            None => panic!("Account not found!"),
            Some(acct) => {
                acct.storage.insert(key.clone(), value.clone());
            }
        }
    }

    fn storage_load(&self, key: &StorageKey) -> Vec<u8> {
        let state = self.state_ref.borrow();
        match &state.current_tx {
            None => panic!("Tx not initialized!"),
            Some(tx) => {
                match state.accounts.get(&tx.to) {
                    None => panic!("Account not found!"),
                    Some(acct) => {
                        match acct.storage.get(key) {
                            None => Vec::with_capacity(0),
                            Some(value) => {
                                value.clone()
                            },
                        }
                    }
                }
            }
        }
    }

    fn storage_store_bytes32(&self, key: &StorageKey, value: &[u8; 32]) {
        let mut vector = Vec::with_capacity(32);
        for i in value.iter() {
            vector.push(*i);
        }
        self.storage_store(key, &vector);
    }
    
    fn storage_load_bytes32(&self, key: &StorageKey) -> [u8; 32] {
        let value = self.storage_load(key);
        let mut res = [0u8; 32];
        let offset = 32 - value.len();
        for i in 0..value.len()-1 {
            res[offset+i] = value[i];
        }
        res
    }

    fn storage_store_big_uint(&self, key: &StorageKey, value: &RustBigUint) {
        self.storage_store(key, &value.to_bytes_be());
    }

    fn storage_load_big_uint(&self, key: &StorageKey) -> RustBigUint {
        let value = self.storage_load(key);
        let bi = BigInt::from_signed_bytes_be(value.as_slice());
        bi.into()
    }

    fn storage_store_big_int(&self, _key: &StorageKey, _value: &RustBigInt) {
        panic!("storage_store_big_int not yet implemented")
    }

    fn storage_load_big_int(&self, _key: &StorageKey) -> RustBigInt {
        panic!("storage_load_big_int not yet implemented")
    }

    fn storage_store_i64(&self, key: &StorageKey, value: i64) {
        self.storage_store_big_int(key, &RustBigInt::from(value));
    }

    fn storage_load_i64(&self, key: &StorageKey) -> Option<i64> {
        let bi = self.storage_load_big_int(key);
        bi.value().to_i64()
    }

    #[inline]
    fn get_call_value_big_uint(&self) -> RustBigUint {
        let state = self.state_ref.borrow();
        match &state.current_tx {
            None => panic!("Tx not initialized!"),
            Some(tx) => tx.call_value.clone().into(),
        }
    }

    fn send_tx(&self, to: &Address, amount: &RustBigUint, _message: &str) {
        let owner = self.get_owner();
        let mut state = self.state_ref.borrow_mut();
        match state.accounts.get_mut(&owner) {
            None => panic!("Account not found!"),
            Some(acct) => {
                acct.balance -= amount.value();
            }
        }
        match state.accounts.get_mut(to) {
            None => panic!("Account not found!"),
            Some(acct) => {
                acct.balance += amount.value();
            }
        }
    }

    fn async_call(&self, _to: &Address, _amount: &RustBigUint, _data: &str) {
        panic!("async_call not yet implemented");
    }

    fn get_gas_left(&self) -> i64 {
        0
    }

    fn sha256(&self, data: &Vec<u8>) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.input(data.as_slice());
        hasher.result().into()
    }

    fn keccak256(&self, data: &Vec<u8>) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.input(data.as_slice());
        hasher.result().into()
    }
}

impl ArwenMockState {
    fn get_argument_vec(&self, arg_index: i32) -> Vec<u8> {
        let arg_idx_usize: usize = arg_index as usize;
        match &self.current_tx {
            None => panic!("Tx not initialized!"),
            Some(tx) => {
                if arg_idx_usize >= tx.args.len() {
                    panic!("Tx arg index out of range");
                }
                tx.args[arg_idx_usize].clone()
            },
        }
    }
}

impl elrond_wasm::ContractIOApi<RustBigInt, RustBigUint> for ArwenMockRef {

    fn get_num_arguments(&self) -> i32 {
        let state = self.state_ref.borrow();
        let nr_args = match &state.current_tx {
            None => panic!("Tx not initialized!"),
            Some(tx) => tx.args.len(),
        };
        nr_args as i32
    }

    fn check_not_payable(&self) -> bool {
        if self.get_call_value_big_uint() > 0 {
            self.signal_error("attempted to transfer funds via a non-payable function");
            return false;
        }
        return true;
    }

    fn get_argument_vec(&self, arg_index: i32) -> Vec<u8> {
        let state = self.state_ref.borrow();
        let arg = state.get_argument_vec(arg_index);
        arg.clone()
    }

    fn get_argument_bytes32(&self, arg_index: i32) -> [u8; 32] {
        let state = self.state_ref.borrow();
        let arg = state.get_argument_vec(arg_index);
        let mut res = [0u8; 32];
        let offset = 32 - arg.len();
        for i in 0..arg.len()-1 {
            res[offset+i] = arg[i];
        }
        res
    }
    
    fn get_argument_big_int(&self, arg_index: i32) -> RustBigInt {
        let state = self.state_ref.borrow();
        let bytes = state.get_argument_vec(arg_index);
        BigInt::from_signed_bytes_be(&bytes).into()
    }

    #[inline]
    fn get_argument_big_uint(&self, _arg_index: i32) -> RustBigUint {
        panic!("get_argument_big_uint not yet implemented")
    }

    #[inline]
    fn get_argument_i64(&self, arg_index: i32) -> i64 {
        let state = self.state_ref.borrow();
        let bytes = state.get_argument_vec(arg_index);
        let bi = BigInt::from_signed_bytes_be(&bytes);
        if let Some(v) = bi.to_i64() {
            v
        } else {
            panic!("Argument does not fit in an i64.")
        }
    }

    fn finish_vec(&self, v: &Vec<u8>) {
        let mut state = self.state_ref.borrow_mut();
        state.add_result(v.clone());
    }

    fn finish_bytes32(&self, bytes: &[u8; 32]) {
        self.finish_vec(&bytes.to_vec());
    }

    #[inline]
    fn finish_big_int(&self, bi: &RustBigInt) {
        let mut state = self.state_ref.borrow_mut();
        state.add_result(bi.to_signed_bytes_be());
    }

    #[inline]
    fn finish_big_uint(&self, bu: &RustBigUint) {
        let mut state = self.state_ref.borrow_mut();
        state.add_result(bu.to_bytes_be());
    }
    
    #[inline]
    fn finish_i64(&self, value: i64) {
        self.finish_big_int(&value.into());
    }

    fn signal_error_raw(&self, message_ptr: *const u8, message_len: usize) {
        let s = unsafe {
            let slice = std::slice::from_raw_parts(message_ptr, message_len);
            std::str::from_utf8(slice)
        };
        panic!("signal_error was called with message: {}", s.unwrap());
    }

    fn write_log(&self, _topics: &[[u8;32]], _data: &[u8]) {
        print!("write_log not yet implemented\n");
    }
}
