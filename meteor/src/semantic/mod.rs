use crate::parser::{
    ast::{Expression, Program},
    span::Spanned,
    symbol::Identifier,
};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ErrorKind {
    #[error("Unknown variable {0}")]
    UnknownVariable(Identifier),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("{kind}: (span:?)")]
pub struct Error<'a> {
    pub kind: ErrorKind,
    pub span: &'a Spanned<Expression>,
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

struct Analyzer<'a> {
    errors: Vec<Error<'a>>,
}

impl Analyzer<'_> {
    fn analyze(&mut self, expr: &Spanned<Expression>) {}
}

pub fn analyze<'a>(program: &'a Program) -> Vec<Error<'a>> {
    Analyzer { errors: vec![] }.analyze_prog(program);
}
