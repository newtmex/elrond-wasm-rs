use super::{context::*, value_interpreter::*, value_raw::*};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::fmt;

pub trait InterpretableFrom<T> {
    fn interpret_from(from: T, context: &InterpreterContext) -> Self;
}

#[derive(Clone, Debug)]
pub struct BytesValue {
    pub value: Vec<u8>,
    pub original: ValueSubTree,
}

impl BytesValue {
    pub fn empty() -> Self {
        BytesValue {
            value: Vec::new(),
            original: ValueSubTree::Str(String::default()),
        }
    }
}

impl From<Vec<u8>> for BytesValue {
    fn from(v: Vec<u8>) -> Self {
        BytesValue {
            value: v,
            original: ValueSubTree::Str(String::default()),
        }
    }
}

impl InterpretableFrom<ValueSubTree> for BytesValue {
    fn interpret_from(from: ValueSubTree, context: &InterpreterContext) -> Self {
        BytesValue {
            value: interpret_subtree(&from, context),
            original: from,
        }
    }
}

impl Default for BytesValue {
    fn default() -> Self {
        Self {
            value: Vec::new(),
            original: ValueSubTree::Str(String::new()),
        }
    }
}

impl fmt::Display for BytesValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.original.fmt(f)
    }
}

#[derive(Debug)]
pub struct BigUintValue {
    pub value: BigUint,
    pub original: ValueSubTree,
}

impl InterpretableFrom<ValueSubTree> for BigUintValue {
    fn interpret_from(from: ValueSubTree, context: &InterpreterContext) -> Self {
        let bytes = interpret_subtree(&from, context);
        BigUintValue {
            value: BigUint::from_bytes_be(&bytes),
            original: from,
        }
    }
}

impl fmt::Display for BigUintValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.original.fmt(f)
    }
}

impl Default for BigUintValue {
    fn default() -> Self {
        BigUintValue {
            original: ValueSubTree::default(),
            value: BigUint::from(0u32),
        }
    }
}

#[derive(Debug, Default)]
pub struct U64Value {
    pub value: u64,
    pub original: ValueSubTree,
}

impl InterpretableFrom<ValueSubTree> for U64Value {
    fn interpret_from(from: ValueSubTree, context: &InterpreterContext) -> Self {
        let bytes = interpret_subtree(&from, context);
        let bu = BigUint::from_bytes_be(&bytes);
        U64Value {
            value: bu.to_u64().unwrap(),
            original: from,
        }
    }
}

impl fmt::Display for U64Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.original.fmt(f)
    }
}
