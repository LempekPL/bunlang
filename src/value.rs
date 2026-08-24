use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Range(i64, i64),
    Struct(Vec<Value>),
    Unit,
    Native(fn(Vec<Value>) -> Result<Value, String>),
}

macro_rules! expected_args {
    ($method:expr, $expected:expr, $got:expr) => {
        if $expected != $got {
            return Err(format!(
                "Method '{}' expects {} arguments got {}",
                $method, $expected, $got
            ));
        }
    };
}

impl Value {
    pub fn is_true(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err("Expected boolean condition, but found a different type".to_string()),
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        match self {
            Value::Float(_) => "Float",
            Value::Int(_) => "Int",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
            Value::Range(_, _) => "Range",
            Value::Unit => "Unit",
            Value::Struct(_) => "Struct",
            Value::Native(_) => "<native fn>",
        }
    }

    pub fn call_method(&self, method: &str, args: Vec<Value>) -> Result<Value, String> {
        match self {
            Value::String(s) => match method {
                "len" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::Float(s.len() as f64))
                }
                "trim" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::String(s.trim().to_string()))
                }
                "to_uppercase" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::String(s.to_uppercase()))
                }
                "to_lowercase" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::String(s.to_lowercase()))
                }
                _ => Err(format!("Unknown method '{}' for String", method)),
            },
            Value::Int(i) => match method {
                "to_float" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::Float(*i as f64))
                }
                "to_string" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::String(i.to_string()))
                }
                _ => Err(format!("Unknown method '{}' for Int", method)),
            },
            Value::Float(f) => match method {
                "to_int" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::Int(*f as i64))
                }
                "to_string" => {
                    expected_args!(method, 0, args.len());
                    Ok(Value::String(f.to_string()))
                }
                _ => Err(format!("Unknown method '{}' for Float", method)),
            },
            other => Err(format!("Type '{}' has no methods", other.get_type_name())),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Float(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Unit => write!(f, "()"),
            Value::Struct(_) => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Range(start, end) => write!(f, "{}..{}", start, end),
            Value::Native(_) => write!(f, "<native fn>"),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs + rhs)),
            (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs + rhs)),
            (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
            (a, b) => Err(format!(
                "Miss matched types, cannot add {:?} and {:?}",
                a, b
            )),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs - rhs)),
            (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs - rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot subtract strings".to_string()),
            (a, b) => Err(format!(
                "Miss matched types, cannot subtract {:?} and {:?}",
                a, b
            )),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs * rhs)),
            (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs * rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot multiply strings".to_string()),
            (a, b) => Err(format!(
                "Miss matched types, cannot multiply {:?} and {:?}",
                a, b
            )),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs / rhs)),
            (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs / rhs)),
            (Value::String(_), Value::String(_)) => Err("Cannot divide strings".to_string()),
            (a, b) => Err(format!(
                "Miss matched types, cannot divide {:?} and {:?}",
                a, b
            )),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, String>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs % rhs)),
            (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs % rhs)),
            (Value::String(_), Value::String(_)) => {
                Err("Cannot use reminder on strings".to_string())
            }
            (a, b) => Err(format!(
                "Miss matched types, cannot reminder {:?} and {:?}",
                a, b
            )),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, String>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Float(num) => Ok(Value::Float(-num)),
            Value::Int(num) => Ok(Value::Int(-num)),
            Value::String(_) => Err("Cannot negate strings".to_string()),
            a => Err(format!("Cannot negate {:?}", a)),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, String>;
    fn not(self) -> Self::Output {
        match self {
            Value::Bool(boolean) => Ok(Value::Bool(!boolean)),
            Value::Int(num) => Ok(Value::Bool(num != 0)),
            a => Err("Cannot not this type".to_string()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Float(lhs), Value::Float(rhs)) => lhs.partial_cmp(rhs),
            (Value::Int(lhs), Value::Int(rhs)) => lhs.partial_cmp(rhs),
            (Value::String(lhs), Value::String(rhs)) => lhs.partial_cmp(rhs),
            (Value::Bool(lhs), Value::Bool(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}
