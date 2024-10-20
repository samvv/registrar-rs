
use crate::{Error, Result};
use serde_json::{Value, Map};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl std::fmt::Display for ValueType {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Null => write!(f, "the null constant"),
            ValueType::Bool => write!(f, "a boolean"),
            ValueType::Number => write!(f, "a number"),
            ValueType::String => write!(f, "a string"),
            ValueType::Array => write!(f, "an array"),
            ValueType::Object => write!(f, "an object"),
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

impl std::fmt::Display for NumberType {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
                NumberType::U64 => write!(f, "an unsigned 64-bit integer"),
                NumberType::I64 => write!(f, "a signed 64-bit integer"),
                NumberType::U32 => write!(f, "an unsigned 32-bit integer"),
                NumberType::Float => write!(f, "a floating point number"),
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
    fn as_object_mut_ok(&mut self) -> Result<&mut Map<String, Value>>;
    fn as_array_ok(&self) -> Result<&Vec<Value>>;
    fn as_array_mut_ok(&mut self) -> Result<&mut Vec<Value>>;
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

    fn as_object_mut_ok(&mut self) -> Result<&mut Map<String, Value>> {
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

    fn as_array_mut_ok(&mut self) -> Result<&mut Vec<Value>> {
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

