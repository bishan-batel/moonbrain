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

    #[must_use]
    pub fn std_include() -> Self {
        Default::default()
    }

    pub fn push(&mut self, name: Spanned<Identifier>) {
        self.symbols.insert(name.0, name.1);
    }

    pub fn contains(&self, ident: &Identifier) -> bool {
        if self.symbols.contains_key(ident) {
            true
        } else {
            match ident.name() {
                "print" => true,
                _ => false,
            }
        }
    }
}
