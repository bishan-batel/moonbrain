use core::fmt;
use std::{
    cell::RefCell,
    ops::{self, Deref, DerefMut, Index},
    rc::Rc,
};

use chumsky::container::Container;
use runtime::Result;

use crate::runtime;

use super::value::{ValConvert, Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct RawArray<T: ValConvert>(Vec<T>);

trait RawArrayErased: fmt::Debug {
    fn push(&mut self, value: Value);

    #[must_use]
    fn set(&mut self, idx: usize, value: Value) -> Option<()>;

    #[must_use]
    fn get(&self, value: Value) -> Option<Value>;

    fn len(&self) -> usize;

    fn clone(&self) -> Box<dyn RawArrayErased>;
}

impl<T: ValConvert + fmt::Debug + Clone + 'static> RawArrayErased for RawArray<T> {
    fn push(&mut self, value: Value) {
        // TODO: error handling
        self.0.push(value);
    }

    fn set(&mut self, idx: usize, value: Value) -> Option<()> {
        todo!()
    }

    fn get(&self, value: Value) -> Option<Value> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn clone(&self) -> Box<dyn RawArrayErased> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Array(Rc<RefCell<Box<dyn RawArrayErased>>>);

impl Array {
    fn new<T: ValConvert>() -> Self {
        Self(Rc::new(RefCell::new(
            Box::new(RawArray(vec![])) as Box<dyn RawArrayErased>
        )))
    }
}
