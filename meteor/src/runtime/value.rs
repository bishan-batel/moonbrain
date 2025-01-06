use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Write},
    rc::Rc,
};

use crate::parser::{
    ast::{self},
    symbol::Identifier,
};

use super::memory::{MemEnviornment, Memory};

#[derive(Debug, Default, Hash, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    Any,
    String,
    Bool,
    Number,
    Nil,
    Dictionary {
        key: Box<Type>,
        value: Box<Type>,
    },
    User(Identifier, Vec<Type>),
    Array(Box<Type>),
}

impl Type {
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
            Type::Any | Type::Nil => Value::Nil,
            Type::String => "".into(),
            Type::Bool => false.into(),
            Type::Number => 0.into(),
            Type::Dictionary { .. } => todo!("Dictionaries are not supported yet"),
            Type::Array(..) => todo!("Arrays are not supported yet"),
            Type::User(..) => todo!("Custom user types are not supported"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    inner: Rc<ast::Function>,
    scope: Memory,
}

impl Function {
    pub fn new(inner: Rc<ast::Function>, scope: Memory) -> Self {
        Self { inner, scope }
    }

    pub fn inner(&self) -> &Rc<ast::Function> {
        &self.inner
    }

    pub fn scope(&self) -> &Memory {
        &self.scope
    }

    pub fn scope_mut(&mut self) -> &mut Memory {
        &mut self.scope
    }

    pub fn set_scope(&mut self, scope: Memory) {
        self.scope = scope;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Bool(bool),
    Number(f64),
    Array(Rc<RefCell<Vec<Value>>>),
    Function(Rc<Function>),
    Dictionary(Rc<HashMap<Identifier, Value>>),
    Nil,
}

pub trait ValConvert: Into<Value> + TryFrom<Value, Error = ()> {}

impl Value {
    pub fn is_type(&self, data_type: &Type) -> bool {
        match (self, data_type) {
            (_, Type::Any)
            | (Value::String(_), Type::String)
            | (Value::Bool(_), Type::Bool)
            | (Value::Number(_), Type::Number)
            | (Value::Nil, Type::Nil) => true,
            _ => false,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Bool(b) => *b,

            Value::Number(b) => *b != 0.,
            Value::Dictionary(..) | Value::Array(_) | Value::Function(_) => true,
            Value::Nil => false,
        }
    }

    pub fn falsey(&self) -> bool {
        return !self.truthy();
    }

    pub fn try_coerce(self, into: &Type) -> Option<Value> {
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

    pub fn get_type(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
            Value::Number(_) => Type::Number,
            Value::Array(..) => Type::Array(Default::default()),
            Value::Function(..) => todo!("Functions have no types"),
            Value::Dictionary(..) => Type::Dictionary {
                key: Default::default(),
                value: Default::default(),
            },
            Value::Nil => Type::Nil,
        }
    }
}

macro_rules! from_value {
    ($from: ty, $variant: ident) => {
        impl From<$from> for Value {
            fn from(value: $from) -> Self {
                Self::$variant(value.into())
            }
        }

        impl TryInto<$from> for Value {
            type Error = ();

            fn try_into(self) -> Result<$from, Self::Error> {
                match self {
                    Value::$variant(n) => Ok(n as $from),
                    _ => Err(()),
                }
            }
        }
    };
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

from_value!(bool, Bool);
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

impl PartialEq for Function {
    // two functions are never equal
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(str) => f.write_str(str),
            Value::Bool(b) => f.write_fmt(format_args!("{b}")),
            Value::Number(n) => f.write_fmt(format_args!("{n}")),
            Value::Array(values) => {
                f.write_char('[')?;

                for val in values.borrow().iter() {
                    val.fmt(f)?;
                    f.write_char(',')?;
                }
                f.write_char(']')
            }
            Value::Dictionary(dict) => {
                f.write_char('{')?;
                for (key, val) in dict.iter() {
                    f.write_fmt(format_args!("{key}: {val}"))?;
                }
                f.write_char('}')
            }
            Value::Function(..) => f.write_str("[function]"),
            Value::Nil => f.write_str("nil"),
        }
    }
}
