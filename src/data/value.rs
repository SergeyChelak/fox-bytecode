use std::{fmt::Display, num::ParseFloatError, rc::Rc};

use crate::{Closure, Func, NativeFn, NativeFunc};

pub type Double = f32;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(Double),
    Bool(bool),
    Text(Rc<String>),
    Fun(Rc<Func>),
    NativeFun(Rc<NativeFunc>),
    Closure(Rc<Closure>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::Text(l), Self::Text(r)) => l == r,
            (Self::Fun(l), Self::Fun(r)) => Rc::ptr_eq(l, r),
            (Self::NativeFun(l), Self::NativeFun(r)) => Rc::ptr_eq(l, r),
            (Self::Closure(l), Self::Closure(r)) => Rc::ptr_eq(l, r),
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(val) => write!(f, "{val}"),
            Value::Number(val) => write!(f, "{val}"),
            Value::Text(val) => write!(f, "{val}"),
            Value::Fun(val) => write!(f, "{val}"),
            Value::NativeFun(val) => write!(f, "{val}"),
            Value::Closure(val) => write!(f, "{}", val.func()),
        }
    }
}

impl Value {
    pub fn closure(func: Rc<Func>) -> Self {
        Value::Closure(Rc::new(Closure::new(func)))
    }

    pub fn native_func(func: NativeFn) -> Self {
        Value::NativeFun(Rc::new(NativeFunc::with(func)))
    }

    pub fn text_from_str(value: &str) -> Self {
        Self::Text(Rc::new(value.to_string()))
    }

    pub fn text_from_string(value: String) -> Self {
        Self::Text(Rc::new(value))
    }

    pub fn number(value: Double) -> Self {
        Self::Number(value)
    }

    pub fn number_from(s: &str) -> Result<Self, ParseFloatError> {
        let value = s.parse::<Double>()?;
        Ok(Self::number(value))
    }

    pub fn as_number(&self) -> Option<Double> {
        match self {
            Value::Number(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<Rc<Func>> {
        match self {
            Value::Fun(func_ref) => Some(func_ref.clone()),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<Rc<String>> {
        match self {
            Value::Text(val) => Some(val.clone()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(value) => *value,
            _ => true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OperationError {
    TypeMismatch,
    DivisionByZero,
}

pub type ValueOperation = fn(&Value, &Value) -> Result<Value, OperationError>;

impl Value {
    pub fn add(a: &Value, b: &Value) -> Result<Value, OperationError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
            (val, Value::Text(x)) => {
                let res = format!("{val}{x}");
                Ok(Value::text_from_string(res))
            }
            (Value::Text(x), val) => {
                let res = format!("{x}{val}");
                Ok(Value::text_from_string(res))
            }
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn subtract(a: &Value, b: &Value) -> Result<Value, OperationError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn multiply(a: &Value, b: &Value) -> Result<Value, OperationError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn divide(a: &Value, b: &Value) -> Result<Value, OperationError> {
        match (a, b) {
            (Value::Number(_), Value::Number(y)) if *y == 0.0 => {
                Err(OperationError::DivisionByZero)
            }
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x / y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn equals(a: &Value, b: &Value) -> Result<Value, OperationError> {
        Ok(Value::Bool(a == b))
    }

    pub fn greater(a: &Value, b: &Value) -> Result<Value, OperationError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x > y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn less(a: &Value, b: &Value) -> Result<Value, OperationError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x < y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equality_text() {
        let text = "abc";
        let a = Value::text_from_str(text);
        let b = Value::text_from_str(text);
        let c = Value::text_from_str("other");
        assert_eq!(a, b);
        assert_ne!(a, c);

        assert_eq!(Value::equals(&a, &b), Ok(Value::Bool(true)));
        assert_eq!(Value::equals(&a, &c), Ok(Value::Bool(false)));
    }
}
