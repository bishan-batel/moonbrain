use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(Rc<str>),
    Bool(bool),
}

macro_rules! impl_from {
    ($ty: ty, $variant: ident) => {
        impl From<$ty> for crate::vm::value::Value {
            fn from(val: $ty) -> Self {
                Self::$variant(val)
            }
        }
    };
}

impl_from!(i64, Integer);
impl_from!(f64, Float);
impl_from!(bool, Bool);
impl_from!(Rc<str>, String);

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl TryInto<i64> for Value {
    type Error = ();

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Value::Integer(i) => Ok(i),
            _ => Err(()),
        }
    }
}
