#![cfg_attr(debug_assertions, allow(dead_code))]

use chumsky::{
    error::Rich,
    input::{Input, Stream},
    span::Span,
    Parser,
};
use logos::Logos;
use parser::{ast::Program, lexer::Token, span, src::SourceId};

pub mod parser;
pub mod runtime;
pub mod semantic;

// pub use runtime::array::Array;

pub fn parse(id: SourceId, source: &str) -> Result<Program, Vec<Rich<'_, Token<'_>, span::Span>>> {
    let token_iter = Token::lexer(source)
        .spanned()
        .map(move |(tok, span)| match tok {
            Ok(tok) => (tok, span::Span::new(id.clone(), span)),
            Err(()) => (Token::Error, span::Span::new(id.clone(), span)),
        });

    let token_stream = Stream::from_iter(token_iter).map(
        span::Span::new(id.clone(), 0..source.len()),
        move |(t, s)| (t, s),
    );

    let parser = parser::parser::program_parser();

    parser.parse(token_stream).into_result()
}
