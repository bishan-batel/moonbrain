use std::collections::{HashMap, HashSet};

use logos::Source;

use crate::parser::{
    span::{Span, Spanned},
    symbol::Identifier,
};

#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: HashMap<Identifier, Span>,
}

impl SymbolTable {
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, name: Spanned<Identifier>) {
        self.symbols.insert(name.0, name.1);
    }

    pub fn contains(&self, name: &Identifier) -> bool {
        self.symbols.contains_key(name)
    }
}
