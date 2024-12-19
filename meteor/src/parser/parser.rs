use chumsky::{input::ValueInput, prelude::*};

use crate::parser::ast::Program;
use crate::parser::{ast::Expression, lexer::Token, operator::Operator};

use super::ast::{self, Directive, Span, Spanned};
use super::symbol::Identifier;

// #[allow(clippy::too_many_lines)]
// #[must_use]
// fn expr_parser<'src, I>(
// ) -> impl Parser<'src, I, Spanned<Expression<'src>>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
// where
//     I: ValueInput<'src, Token = Token<'src>, Span = Span>,
// {
//     todo!()
// recursive(|expr| {
//     // number and string literals
//     let literal = select! {
//         Token::Bool(b) => Expression::Bool(b.parse().unwrap()),
//         Token::Number(n)=> Expression::Number(n.parse().unwrap()),
//         Token::Identifier(name) => Expression::Ident(name.into()),
//         Token::String(name) => Expression::String(name.into())
//     }
//     .labelled("value");
//
//     let ident = select! { Token::Identifier(ident) => ident }.labelled("identifier");
//
//     let items = expr
//         .clone()
//         .separated_by(just(Token::Comma))
//         .allow_trailing()
//         .collect::<Vec<_>>();
//
//     let list = items
//         .clone()
//         .map(Expression::Array)
//         .delimited_by(just(Token::BracketOpen), just(Token::BracketClose));
//
//     let atom = literal
//         .or(ident.map(|x| Expression::Ident(x.into())))
//         .or(list)
//         .map_with(|expr, e| (expr, e.span()))
//         // Atoms can also just be normal expressions, but surrounded with parentheses
//         .or(expr
//             .clone()
//             .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed)))
//         // Attempt to recover anything that looks like a parenthesised expression but contains errors
//         .recover_with(via_parser(nested_delimiters(
//             Token::ParenOpen,
//             Token::ParenClosed,
//             [
//                 (Token::BracketOpen, Token::BracketClose),
//                 (Token::CurlyBraceOpen, Token::CurlyBraceClose),
//             ],
//             |span| (Expression::Error, span),
//         )))
//         // Attempt to recover anything that looks like a list but contains errors
//         .recover_with(via_parser(nested_delimiters(
//             Token::BracketOpen,
//             Token::BracketClose,
//             [
//                 (Token::ParenOpen, Token::ParenClosed),
//                 (Token::CurlyBraceOpen, Token::CurlyBraceClose),
//             ],
//             |span| (Expression::Error, span),
//         )))
//         .boxed();
//
//     list
// })
// }

#[cfg(test)]
mod tests {

    // mod expr {
    //     use chumsky::{input::Stream, prelude::*};
    //     use logos::Logos;
    //
    //     use crate::parser::symbol::Identifier;
    //     use crate::parser::{
    //         ast::Expression, lexer::Token, operator::Operator, parser::parse_expr,
    //     };
    //
    //     fn parse(source: &str) -> Result<Expression, Vec<Rich<'_, Token<'_>>>> {
    //         let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
    //             Ok(tok) => (tok, span.into()),
    //             Err(()) => (Token::Error, span.into()),
    //         });
    //
    //         let token_stream = Stream::from_iter(token_iter)
    //             // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
    //             // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
    //             .map((0..source.len()).into(), |(t, s): (_, _)| (t, s));
    //
    //         parse_expr().parse(token_stream).into_result()
    //     }
    //
    //     #[test]
    //     fn simple_expr() {
    //         let a = parse("1 + 2").unwrap();
    //
    //         assert_eq!(
    //             a,
    //             Expression::BinaryOp {
    //                 operator: Operator::Add,
    //                 lhs: Box::new(Expression::Number(1.)),
    //                 rhs: Box::new(Expression::Number(2.))
    //             }
    //         );
    //     }
    //
    //     #[test]
    //     fn call_expr() {
    //         let a = parse("me").unwrap();
    //
    //         assert_eq!(a, Expression::Ident("me".into()));
    //
    //         let a = parse("me()").unwrap();
    //         assert_eq!(
    //             a,
    //             Expression::Call {
    //                 lhs: Box::new(Expression::Ident("me".into())),
    //                 arguments: Vec::new()
    //             }
    //         );
    //
    //         let a = parse("me(true, false)").unwrap();
    //         assert_eq!(
    //             a,
    //             Expression::Call {
    //                 lhs: Box::new(Expression::Ident("me".into())),
    //                 arguments: vec![Expression::Bool(true), Expression::Bool(false)]
    //             }
    //         );
    //     }
    // }
    //
    // mod prog {
    //     use crate::parser::{
    //         ast::{Directive, Program},
    //         parser::parse_program,
    //     };
    //     use chumsky::{input::Stream, prelude::*};
    //     use logos::Logos;
    //
    //     use crate::parser::{ast::Expression, lexer::Token, operator::Operator};
    //
    //     fn parse(source: &str) -> Result<Program, Vec<Rich<'_, Token<'_>>>> {
    //         let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
    //             Ok(tok) => (tok, span.into()),
    //             Err(()) => (Token::Error, span.into()),
    //         });
    //
    //         let token_stream = Stream::from_iter(token_iter)
    //             .map((0..source.len()).into(), |(t, s): (_, _)| (t, s));
    //
    //         parse_program().parse(token_stream).into_result()
    //     }
    //
    //     #[test]
    //     fn test_directives() {
    //         assert_eq!(
    //             parse(r"@inputs()"),
    //             Ok(Program::new(vec![Directive::new("inputs", vec![])], vec![]))
    //         );
    //
    //         assert_eq!(
    //             parse(r#"@inputs(what, "huh")"#),
    //             Ok(Program::new(
    //                 vec![Directive::new(
    //                     "inputs",
    //                     vec![
    //                         Expression::Ident("what".into()),
    //                         Expression::String("huh".into())
    //                     ]
    //                 )],
    //                 vec![]
    //             ))
    //         );
    //     }
    // }
}
