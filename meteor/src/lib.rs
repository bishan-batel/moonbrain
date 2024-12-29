#![cfg_attr(debug_assertions, allow(dead_code))]

use chumsky::{
    error::Rich,
    input::{Input, Stream},
    Parser,
};
use logos::Logos;
use parser::{
    ast::{Expression, Program},
    lexer::Token,
};

pub mod parser;
pub mod runtime;

pub fn parse(source: &str) -> Result<Program, Vec<Rich<'_, Token<'_>>>> {
    let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Token::Error, span.into()),
    });

    let token_stream =
        Stream::from_iter(token_iter).map((0..source.len()).into(), |(t, s): (_, _)| (t, s));

    parser::parser::program_parser()
        .parse(token_stream)
        .into_result()
}
