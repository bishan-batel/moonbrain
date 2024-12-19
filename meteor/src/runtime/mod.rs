use std::sync::Arc;
pub mod io;
pub mod value;

use io::{Sink, Socket};

use crate::parser::{
    ast::{Program, Spanned},
    symbol::Identifier,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", name.0)]
    UnknownVariable { name: Spanned<Identifier> },
}

#[derive(Debug)]
pub struct Chip {
    inputs: Vec<Sink>,
    outputs: Vec<Socket>,
    program: Program,
}

impl Chip {
    pub fn new(program: Program) -> Self {
        todo!()
    }
}
