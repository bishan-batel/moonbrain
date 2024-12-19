use std::{fmt::Display, ops::Deref, sync::Arc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Identifier(Arc<str>);

impl Identifier {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.name(), f)
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
