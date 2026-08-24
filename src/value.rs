use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};
use crate::error::BunError;
use crate::location::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    F64(f64),
    String(String),
    Bool(bool),
    Range(f64, f64),
    Unit,
    Struct(Vec<Value>),
}

impl Value {
    pub fn is_true(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err("Expected boolean condition, but found a different type".to_string()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::F64(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Unit => write!(f, "()"),
            Value::Struct(_) => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Range(start, end) => write!(f, "{}..{}", start, end),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs + rhs)),
            (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
            (a, b) => Err(format!("Miss matched types, cannot add {:?} and {:?}", a, b)),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs - rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot subtract strings".to_string()),
            (a, b) => Err(format!("Miss matched types, cannot add {:?} and {:?}", a, b)),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs * rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot multiply strings".to_string()),
            (a, b) => Err(format!("Miss matched types, cannot add {:?} and {:?}", a, b)),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs / rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot divide strings".to_string()),
            (a, b) => Err(format!("Miss matched types, cannot add {:?} and {:?}", a, b)),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, String>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs % rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot use reminder on strings".to_string()),
            (a, b) => Err(format!("Miss matched types, cannot add {:?} and {:?}", a, b)),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, String>;

    fn neg(self) -> Self::Output {
        match self {
            Value::F64(num) => Ok(Value::F64(-num)),
            Value::String(_) => Err("Cannot negate strings".to_string()),
            a => Err("Cannot negate this type".to_string()),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, String>;
    fn not(self) -> Self::Output {
        match self {
            Value::Bool(boolean) => Ok(Value::Bool(!boolean)),
            a => Err("Cannot not this type".to_string()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::F64(lhs), Value::F64(rhs)) => lhs.partial_cmp(rhs),
            (Value::String(lhs), Value::String(rhs)) => lhs.partial_cmp(rhs),
            (Value::Bool(lhs), Value::Bool(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}