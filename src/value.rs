use crate::bitstring::BitString;
use crate::callable::Callable;

pub enum Value {
    BitString(BitString),
    Callable(Callable),
}

impl Value {
    pub fn into_bit_string(self) -> Option<BitString> {
        match self {
            Value::BitString(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_callable(self) -> Option<Callable> {
        match self {
            Value::Callable(c) => Some(c),
            _ => None,
        }
    }
}

impl From<BitString> for Value {
    fn from(s: BitString) -> Value {
        Value::BitString(s)
    }
}

impl From<Callable> for Value {
    fn from(c: Callable) -> Value {
        Value::Callable(c)
    }
}
