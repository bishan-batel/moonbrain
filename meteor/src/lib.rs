#![cfg_attr(debug_assertions, allow(dead_code))]

use chumsky::{
    error::Rich,
    input::{Input, Stream},
    span::Span,
    Parser,
};
use logos::Logos;
use parser::{
    ast::{Expression, Program},
    lexer::Token,
    span,
    src::SourceId,
};

pub mod parser;
pub mod runtime;

pub fn parse(id: SourceId, source: &str) -> Result<Program, Vec<Rich<'_, Token<'_>, span::Span>>> {
    let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok, span::Span::new(SourceId::new("_"), span)),
        Err(()) => (Token::Error, span::Span::new(SourceId::new("_"), span)),
    });

    let token_stream = Stream::from_iter(token_iter).map(
        span::Span::new(id.clone(), 0..source.len()),
        move |(t, s)| (t, span::Span::new(id, s.range())),
    );

    let parser = parser::parser::program_parser();

    parser.parse(token_stream).into_result()
}
