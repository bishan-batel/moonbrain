use std::{borrow::Borrow, cell::RefCell, collections::HashMap, mem, ops::Deref, rc::Rc};

use dashmap::DashMap;
use serde::Serialize;

use crate::{
    parser::{ast::Expression, span::Spanned, symbol::Identifier},
    runtime::Result,
};

use super::{
    error::RuntimeError,
    value::{Type, Value},
};

#[derive(Debug, Clone)]
pub struct Memory {
    enviornment: Rc<RefCell<MemEnviornment>>,
}

#[derive(Debug)]
pub struct MemEnviornment {
    variables: HashMap<Identifier, Variable>,
    enclosing: Option<Rc<RefCell<MemEnviornment>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub data_type: Type,
    pub mutability: Mutability,
    pub value: Value,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Constant,
    Mutable,
    DeferInit,
}

impl Memory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            enviornment: Rc::new(MemEnviornment::new(None).into()),
        }
    }

    #[must_use]
    pub fn define(
        &mut self,
        ident: Identifier,
        var: Variable,
        expr: &Spanned<Expression>,
    ) -> Result<()> {
        RefCell::borrow_mut(&self.enviornment).define(ident, var, expr)
    }

    #[must_use]
    pub fn store(
        &mut self,
        ident: &Identifier,
        value: Value,
        expr: &Spanned<Expression>,
    ) -> Result<()> {
        RefCell::borrow_mut(&self.enviornment).store(ident, value, expr)
    }

    #[must_use]
    pub fn retrieve(&self, var: &Identifier, expr: &Spanned<Expression>) -> Result<Variable> {
        RefCell::borrow(&self.enviornment).retrieve(var, expr)
    }

    #[must_use]
    pub fn current(&self) -> &Rc<RefCell<MemEnviornment>> {
        &self.enviornment
    }

    pub fn push_env(&mut self) {
        let curr = self.enviornment.clone();
        self.enviornment = Rc::new(MemEnviornment::new(Some(curr)).into());
    }

    pub fn pop_env(&mut self) -> Rc<RefCell<MemEnviornment>> {
        let enclosing = self
            .enviornment
            .borrow_mut()
            .enclosing
            .clone()
            .expect("Cannot pop off global enviornment");

        mem::replace(&mut self.enviornment, enclosing)
    }
}

impl MemEnviornment {
    #[must_use]
    fn new(enclosing: Option<Rc<RefCell<Self>>>) -> Self {
        Self {
            variables: Default::default(),
            enclosing,
        }
    }

    #[must_use]
    pub fn define(
        &mut self,
        ident: Identifier,
        var: Variable,
        _expr: &Spanned<Expression>,
    ) -> Result<()> {
        let _ = self.variables.insert(ident, var);
        Ok(())
    }

    #[must_use]
    pub fn store(
        &mut self,
        ident: &Identifier,
        value: Value,
        expr: &Spanned<Expression>,
    ) -> Result<()> {
        if let Some(var) = self.variables.get_mut(ident) {
            return if var.value.is_type(var.data_type()) {
                var.value = value;
                Ok(())
            } else {
                Err(RuntimeError::MismatchType {
                    name: ident.clone(),
                    data_type: var.data_type.clone(),
                    expr: expr.clone(),
                })
            };
        }

        match self.enclosing.as_ref() {
            Some(enclosing) => RefCell::borrow_mut(enclosing).store(ident, value, expr),
            _ => Err(RuntimeError::UnknownVariable {
                name: ident.clone(),
                expr: expr.clone(),
            }),
        }
    }

    #[must_use]
    pub fn retrieve(&self, name: &Identifier, expr: &Spanned<Expression>) -> Result<Variable> {
        if let Some(var) = self.variables.get(name).map(Variable::clone) {
            return Ok(var);
        }

        match self.enclosing.as_ref() {
            Some(enclosing) => RefCell::borrow(enclosing).retrieve(name, expr),
            _ => Err(RuntimeError::UnknownVariable {
                name: name.clone(),
                expr: expr.clone(),
            }),
        }
    }
}

impl Variable {
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn data_type(&self) -> &Type {
        &self.data_type
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }
}
