use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use crate::error::BunError;
use crate::location::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    F64(f64),
    String(String),
    Unit,
    Struct(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::F64(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Unit => write!(f, "()"),
            Value::Struct(_) => write!(f, "()"),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, BunError<'static>>;

    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs + rhs)),
            (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
            (a, b) => Err(BunError {
                message: format!("Miss matched types, cannot add {:?} and {:?}", a, b),
                span: Span::default(),
                source: "",
            }),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, BunError<'static>>;

    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs - rhs)),
            (Value::String(_), Value::String(_)) => Err(BunError {
                message: format!("Cannot subtract strings"),
                span: Span::default(),
                source: "",
            }),
            (a, b) => Err(BunError {
                message: format!("Miss matched types, cannot add {:?} and {:?}", a, b),
                span: Span::default(),
                source: "",
            }),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, BunError<'static>>;

    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs * rhs)),
            (Value::String(_), Value::String(_)) => Err(BunError {
                message: format!("Cannot multiply strings"),
                span: Span::default(),
                source: "",
            }),
            (a, b) => Err(BunError {
                message: format!("Miss matched types, cannot add {:?} and {:?}", a, b),
                span: Span::default(),
                source: "",
            }),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, BunError<'static>>;

    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs / rhs)),
            (Value::String(_), Value::String(_)) => Err(BunError {
                message: format!("Cannot divide strings"),
                span: Span::default(),
                source: "",
            }),
            (a, b) => Err(BunError {
                message: format!("Miss matched types, cannot add {:?} and {:?}", a, b),
                span: Span::default(),
                source: "",
            }),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, BunError<'static>>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::F64(lhs), Value::F64(rhs)) => Ok(Value::F64(lhs / rhs)),
            (Value::String(_), Value::String(_)) => Err(BunError {
                message: "Cannot use reminder on strings".to_string(),
                span: Span::default(),
                source: "",
            }),
            (a, b) => Err(BunError {
                message: format!("Miss matched types, cannot add {:?} and {:?}", a, b),
                span: Span::default(),
                source: "",
            }),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, BunError<'static>>;

    fn neg(self) -> Self::Output {
        match self {
            Value::F64(num) => Ok(Value::F64(-num)),
            Value::String(_) => Err(BunError {
                message: "Cannot negate strings".to_string(),
                span: Span::default(),
                source: "",
            }),
            a => Err(BunError {
                message: "Cannot negate this type".to_string(),
                span: Span::default(),
                source: "",
            }),
        }
    }
}