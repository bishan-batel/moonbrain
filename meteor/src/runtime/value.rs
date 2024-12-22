

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeInfo {
    Any,
    String,
    Bool,
    Number,
    Nil,
    Dictionary {
        key: Box<TypeInfo>,
        value: Box<TypeInfo>,
    },
    Array(Box<TypeInfo>),
}

impl TypeInfo {
    pub fn array() -> Self {
        Self::Array(Box::new(Self::Any))
    }

    pub fn dict() -> Self {
        Self::Dictionary {
            key: Box::new(Self::Any),
            value: Box::new(Self::Any),
        }
    }

    pub fn default(&self) -> Value {
        match self {
            TypeInfo::Any | TypeInfo::Nil => Value::Nil,
            TypeInfo::String => "".into(),
            TypeInfo::Bool => false.into(),
            TypeInfo::Number => 0.into(),
            TypeInfo::Dictionary { .. } => todo!("Dictionaries are not supported yet"),
            TypeInfo::Array(..) => todo!("Arrays are not supported yet"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Bool(bool),
    Number(f64),
    Nil,
}

impl Value {
    pub fn is_type(&self, data_type: &TypeInfo) -> bool {
        match (self, data_type) {
            (_, TypeInfo::Any)
            | (Value::String(_), TypeInfo::String)
            | (Value::Bool(_), TypeInfo::Bool)
            | (Value::Number(_), TypeInfo::Number)
            | (Value::Nil, TypeInfo::Nil) => true,
            _ => false,
        }
    }

    pub fn try_coerce(self, into: &TypeInfo) -> Option<Value> {
        if self.is_type(into) {
            return Some(self);
        }

        Some(match (self, into.default()) {
            // bools can be converted to numbers
            (Self::Bool(b), Self::Number(_)) => Self::Number(if b { 1. } else { 0. }),

            // anything can convert to string
            (val, Self::String(_)) => Self::String(format!("{val:?}")),

            // anything can coerce to Nil
            (_, Self::Nil) => Self::Nil,
            _ => return None,
        })
    }
}

macro_rules! from_value {
    ($from: ty, $variant: ident) => {
        impl From<$from> for Value {
            fn from(value: $from) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}

from_value!(bool, Bool);
from_value!(&str, String);
from_value!(String, String);
from_value!(i8, Number);
from_value!(i16, Number);
from_value!(i32, Number);
from_value!(u8, Number);
from_value!(u16, Number);
from_value!(u32, Number);
from_value!(f32, Number);
from_value!(f64, Number);

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Nil
    }
}
