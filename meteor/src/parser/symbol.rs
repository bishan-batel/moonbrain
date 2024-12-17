use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(Arc<str>);

impl Identifier {
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
