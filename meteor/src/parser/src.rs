use core::fmt;
use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use chumsky::span::SimpleSpan;
use internment::Intern;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Sourced<T: ?Sized> {
    inner: Box<T>,
    span: SimpleSpan,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceId(Intern<String>);

impl SourceId {
    pub fn empty() -> Self {
        Self(Intern::new("[test]".into()))
    }

    pub fn new(str: impl Into<String>) -> Self {
        Self(Intern::new(str.into()))
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Sourced<T> {
    pub fn new(inner: T, span: SimpleSpan) -> Self {
        Self {
            inner: Box::new(inner),
            span,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn map<U, F>(self, func: F) -> Sourced<U>
    where
        F: FnOnce(T) -> U,
    {
        Sourced {
            inner: Box::new(func(*self.inner)),
            span: self.span,
        }
    }

    pub fn as_mut(&mut self) -> (&mut T, &mut SimpleSpan) {
        (&mut self.inner, &mut self.span)
    }

    pub fn into_inner(self) -> T {
        *self.inner
    }
}

impl<T> Deref for Sourced<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> DerefMut for Sourced<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<T> PartialEq for Sourced<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: PartialOrd> PartialOrd for Sourced<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T: fmt::Debug> fmt::Debug for Sourced<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?} @ {:?}", self.inner, self.span)
    }
}
