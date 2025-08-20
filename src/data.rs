use std::{fmt::Display, num::ParseFloatError};

pub type Double = f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    Nil,
    Number(Double),
    Bool(bool),
}

impl Default for DataType {
    fn default() -> Self {
        Self::Nil
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Nil => write!(f, "nil"),
            DataType::Bool(val) => write!(f, "{val}"),
            DataType::Number(val) => write!(f, "{val}"),
        }
    }
}

impl DataType {
    pub fn number(value: Double) -> Self {
        Self::Number(value)
    }

    pub fn number_from(s: &str) -> Result<Self, ParseFloatError> {
        let value = s.parse::<Double>()?;
        Ok(Self::number(value))
    }

    pub fn as_number(&self) -> Option<Double> {
        match self {
            DataType::Number(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            DataType::Nil => false,
            DataType::Bool(value) => *value,
            _ => true,
        }
    }
}
