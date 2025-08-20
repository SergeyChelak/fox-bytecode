use std::{fmt::Display, num::ParseFloatError};

pub type Double = f32;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Nil,
    Number(Double),
    Bool(bool),
    Text(String),
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
            DataType::Text(val) => write!(f, "{val}"),
        }
    }
}

impl DataType {
    pub fn str_text(value: &str) -> Self {
        Self::Text(value.to_string())
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

#[derive(Debug, PartialEq)]
pub enum OperationError {
    TypeMismatch,
    DivisionByZero,
}

pub type DataOperation = fn(&DataType, &DataType) -> Result<DataType, OperationError>;

impl DataType {
    pub fn add(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        match (a, b) {
            (DataType::Number(x), DataType::Number(y)) => Ok(DataType::Number(x + y)),
            (DataType::Text(x), DataType::Text(y)) => {
                let mut res = String::new();
                res.push_str(x.as_str());
                res.push_str(y.as_str());
                Ok(DataType::Text(res))
            }
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn subtract(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        match (a, b) {
            (DataType::Number(x), DataType::Number(y)) => Ok(DataType::Number(x - y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn multiply(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        match (a, b) {
            (DataType::Number(x), DataType::Number(y)) => Ok(DataType::Number(x * y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn divide(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        match (a, b) {
            (DataType::Number(_), DataType::Number(y)) if *y == 0.0 => {
                Err(OperationError::DivisionByZero)
            }
            (DataType::Number(x), DataType::Number(y)) => Ok(DataType::Number(x / y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn equals(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        Ok(DataType::Bool(a == b))
    }

    pub fn greater(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        match (a, b) {
            (DataType::Number(x), DataType::Number(y)) => Ok(DataType::Bool(x > y)),
            _ => Err(OperationError::TypeMismatch),
        }
    }

    pub fn less(a: &DataType, b: &DataType) -> Result<DataType, OperationError> {
        match (a, b) {
            (DataType::Number(x), DataType::Number(y)) => Ok(DataType::Bool(x < y)),
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
        let a = DataType::str_text(text);
        let b = DataType::str_text(text);
        let c = DataType::str_text("other");
        assert_eq!(a, b);
        assert_ne!(a, c);

        assert_eq!(DataType::equals(&a, &b), Ok(DataType::Bool(true)));
        assert_eq!(DataType::equals(&a, &c), Ok(DataType::Bool(false)));
    }
}
