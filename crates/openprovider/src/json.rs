
use serde_json::{Value, Map, Number};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl ValueType {
    fn describe(&self) -> String {
        match self {
            ValueType::Null => "the null constant".to_string(),
            ValueType::Bool => "a boolean".to_string(),
            ValueType::Number => "a number".to_string(),
            ValueType::String => "a string".to_string(),
            ValueType::Array => "an array".to_string(),
            ValueType::Object => "an object".to_string(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NumberType {
    I64,
    U64,
    U32,
    Float,
}

#[derive(Debug)]
pub enum Error {
    IndexOutOfBounds(usize),
    KeyMissing(String),
    WrongType(Value, ValueType),
    WrongNumberType(Number, NumberType),
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyMissing(key) => write!(f, "key '{}' is missing", key),
            Self::WrongType(_value, expected) => write!(f, "expected {}", expected.describe()),
            Self::WrongNumberType(_value, expected) => write!(f, "expected {}", match expected {
                NumberType::U64 => "an unsigned 64-bit integer",
                NumberType::I64 => "a signed 64-bit integer",
                NumberType::U32 => "an unsigned 32-bit integer",
                NumberType::Float => "a floating point number",
            }),
            Self::IndexOutOfBounds(index) => write!(f, "index {} is out of bounds", index),
        }
    }
}

pub trait IndexExt {
    fn index_into<'v>(&self, v: &'v Value) -> Result<&'v Value>;
}

impl IndexExt for usize {
    fn index_into<'v>(&self, v: &'v Value) -> Result<&'v Value> {
        match v {
            Value::Array(vec) => 
                match vec.get(*self) {
                    None => Err(Error::IndexOutOfBounds(*self)),
                    Some(value) => Ok(value),
                },
            _ => Err(Error::WrongType(v.clone(), ValueType::Array)),
        }
    }
}

impl IndexExt for str {

    fn index_into<'v>(&self, v: &'v Value) -> Result<&'v Value> {
        match v {
            Value::Object(map) =>
                match map.get(self) {
                    None => Err(Error::KeyMissing(self.to_string())),
                    Some(value) => Ok(value),
                },
            _ => Err(Error::WrongType(v.clone(), ValueType::Object)),
        }
    }

}

impl<'a, T> IndexExt for &'a T
where
    T: ?Sized + IndexExt,
{
    fn index_into<'v>(&self, v: &'v Value) -> Result<&'v Value> {
        (**self).index_into(v)
    }
}

pub trait ValueExt {
    fn get_ok<I: IndexExt>(&self, index: I) -> Result<&Value>;
    fn as_object_ok(&self) -> Result<&Map<String, Value>>;
    fn as_array_ok(&self) -> Result<&Vec<Value>>;
    fn as_str_ok(&self) -> Result<&str>;
    fn as_bool_ok(&self) -> Result<bool>;
    fn as_null_ok(&self) -> Result<()>;
    fn as_i64_ok(&self) -> Result<i64>;
    fn as_u64_ok(&self) -> Result<u64>;
    fn as_f64_ok(&self) -> Result<f64>;
    fn as_u32_ok(&self) -> Result<u32>;
}

impl ValueExt for serde_json::Value {

    fn get_ok<I: IndexExt>(&self, index: I) -> Result<&Value> {
        index.index_into(self)
    }

    fn as_object_ok(&self) -> Result<&Map<String, Value>> {
        match self {
            Value::Object(map) => Ok(map),
            _ => Err(Error::WrongType(self.clone(), ValueType::Object)),
        }
    }

    fn as_array_ok(&self) -> Result<&Vec<Value>> {
        match self {
            Value::Array(vec) => Ok(vec),
            _ => Err(Error::WrongType(self.clone(), ValueType::Array)),
        }
    }

    fn as_str_ok(&self) -> Result<&str> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(Error::WrongType(self.clone(), ValueType::String)),
        }
    }

    fn as_bool_ok(&self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(Error::WrongType(self.clone(), ValueType::Bool)),
        }
    }

    fn as_null_ok(&self) -> Result<()> {
        match self {
            Value::Null => Ok(()),
            _ => Err(Error::WrongType(self.clone(), ValueType::Null))
        }
    }

    fn as_u64_ok(&self) -> Result<u64> {
        match self {
            Value::Number(num) if num.is_u64() => Ok(num.as_u64().unwrap()),
            Value::Number(num) => Err(Error::WrongNumberType(num.clone(), NumberType::U64)),
            _ => Err(Error::WrongType(self.clone(), ValueType::Number))
        }
    }

    fn as_u32_ok(&self) -> Result<u32> {
        match self {
            Value::Number(num) if num.is_u64() && num.as_u64().unwrap() < (std::u32::MAX as u64) => Ok(num.as_u64().unwrap() as u32),
            Value::Number(num) if num.is_u64() => Err(Error::WrongNumberType(num.clone(), NumberType::U32)),
            Value::Number(num) => Err(Error::WrongNumberType(num.clone(), NumberType::U64)),
            _ => Err(Error::WrongType(self.clone(), ValueType::Number))
        }
    }

    fn as_i64_ok(&self) -> Result<i64> {
        match self {
            Value::Number(num) if num.is_i64() => Ok(num.as_i64().unwrap()),
            Value::Number(num) => Err(Error::WrongNumberType(num.clone(), NumberType::I64)),
            _ => Err(Error::WrongType(self.clone(), ValueType::Number))
        }
    }

    fn as_f64_ok(&self) -> Result<f64> {
        match self {
            Value::Number(num) if num.is_f64() => Ok(num.as_f64().unwrap()),
            Value::Number(num) => Err(Error::WrongNumberType(num.clone(), NumberType::Float)),
            _ => Err(Error::WrongType(self.clone(), ValueType::Number))
        }
    }

}

