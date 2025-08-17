use std::{fmt::Display, num::ParseFloatError};

pub type Double = f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Number(Double),
    Bool(bool),
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(val) => write!(f, "{val}"),
            Value::Number(val) => write!(f, "{val}"),
        }
    }
}

impl Value {
    pub fn number(value: Double) -> Self {
        Self::Number(value)
    }

    pub fn number_from(s: &str) -> Result<Self, ParseFloatError> {
        let value = s.parse::<Double>()?;
        Ok(Self::number(value))
    }

    // fn is_nil(&self) -> bool {
    //     matches!(self, Value::Nil)
    // }

    // fn is_number(&self) -> bool {
    //     matches!(self, Value::Number(_))
    // }

    // fn is_boolean(&self) -> bool {
    //     matches!(self, Value::Bool(_))
    // }

    pub fn as_number(&self) -> Option<Double> {
        match self {
            Value::Number(x) => Some(*x),
            _ => None,
        }
    }
}
