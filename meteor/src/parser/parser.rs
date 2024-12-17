use chumsky::{combinator::Foldr, input::ValueInput, prelude::*};

use crate::parser::{
    ast::{Expression, Program},
    lexer::Token,
    operator::Operator,
};

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn parse_expr<'a, I>() -> impl Parser<'a, I, Expression, extra::Err<Rich<'a, Token<'a>>>>
where
    I: ValueInput<'a, Token = Token<'a>, Span = SimpleSpan>,
{
    recursive(|expr| {
        // number and string literals
        let literal = select! {
            Token::Bool(b) => Expression::Bool(b.parse().unwrap()),
            Token::Number(n)=> Expression::Number(n.parse().unwrap()),
            Token::Identifier(name) => Expression::Ident(name.into()),
            Token::String(name) => Expression::String(name.into())
        };

        // let call = expr
        //     .clone()
        //     .then(
        //         expr.clone()
        //             .separated_by(just(Token::Comma))
        //             .collect::<Vec<_>>()
        //             .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed)),
        //     )
        //     .map(|(x, args)| Expression::Call {
        //         lhs: Box::new(x),
        //         arguments: args,
        //     });

        let items = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let atom = literal
            .or(expr.delimited_by(just(Token::ParenOpen), just(Token::ParenClosed)))
            // .or(call)
            .boxed();

        let call = atom
            .foldl_with(
                items
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed))
                    .map_with(|args, e| (args, e.span()))
                    .repeated(),
                |f, args, e| Expression::Call {
                    lhs: Box::new(f),
                    arguments: args.0,
                },
            )
            .boxed();

        let op = |operator: Operator| select! { Token::Operator(op) if op == operator => op};

        let unary = choice((op(Operator::Sub), op(Operator::Not)))
            .repeated()
            .foldr(call, |o, rhs| Expression::UnaryOp {
                operator: o,
                rhs: Box::new(rhs),
            })
            .boxed();

        let binary = |operator| {
            op(operator).to(move |lhs, rhs| Expression::BinaryOp {
                operator,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        };

        let product = unary
            .clone()
            .foldl(
                choice((
                    binary(Operator::Mul),
                    binary(Operator::Div),
                    binary(Operator::Mod),
                ))
                .then(unary)
                .repeated(),
                |lhs, (op, rhs)| op(lhs, rhs),
            )
            .boxed();

        let sum = product
            .clone()
            .foldl(
                choice((binary(Operator::Add), binary(Operator::Sub)))
                    .then(product)
                    .repeated(),
                |lhs, (op, rhs)| op(lhs, rhs),
            )
            .boxed();

        let xor = sum.clone().foldl(
            binary(Operator::Xor).then(sum).repeated(),
            |lhs, (op, rhs)| op(lhs, rhs),
        );

        let comparisons = xor
            .clone()
            .foldl(
                choice((
                    binary(Operator::Equals),
                    binary(Operator::NotEqual),
                    binary(Operator::Greater),
                    binary(Operator::GreaterOrEqual),
                    binary(Operator::Less),
                    binary(Operator::LessOrEqual),
                ))
                .then(xor)
                .repeated(),
                |lhs, (op, rhs)| op(lhs, rhs),
            )
            .boxed();

        let and_expr = comparisons.clone().foldl(
            binary(Operator::And).then(comparisons).repeated(),
            |lhs, (op, rhs)| op(lhs, rhs),
        );

        let ors = and_expr.clone().foldl(
            choice((binary(Operator::Or), binary(Operator::Nor)))
                .then(and_expr)
                .repeated(),
            |lhs, (op, rhs)| op(lhs, rhs),
        );

        let assignments = ors
            .clone()
            .foldl(
                binary(Operator::Assign).then(ors).repeated(),
                |lhs, (op, rhs)| op(lhs, rhs),
            )
            .boxed();

        assignments
    })
}

#[cfg(test)]
mod tests {
    use chumsky::{input::Stream, prelude::*};
    use logos::Logos;

    use crate::parser::{ast::Expression, lexer::Token, operator::Operator, symbol::Identifier};

    use super::parse_expr;

    fn parse(source: &str) -> Result<Expression, Vec<Rich<'_, Token<'_>>>> {
        let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
            Ok(tok) => (tok, span.into()),
            Err(()) => (Token::Error, span.into()),
        });

        let token_stream = Stream::from_iter(token_iter)
            // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
            // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
            .map((0..source.len()).into(), |(t, s): (_, _)| (t, s));

        parse_expr().parse(token_stream).into_result()
    }

    #[test]
    fn simple_expr() {
        let a = parse("1 + 2").unwrap();

        assert_eq!(
            a,
            Expression::BinaryOp {
                operator: Operator::Add,
                lhs: Box::new(Expression::Number(1.)),
                rhs: Box::new(Expression::Number(2.))
            }
        );
    }

    #[test]
    fn call_expr() {
        let a = parse("me").unwrap();

        assert_eq!(a, Expression::Ident("me".into()));

        let a = parse("me()").unwrap();
        assert_eq!(
            a,
            Expression::Call {
                lhs: Box::new(Expression::Ident("me".into())),
                arguments: Vec::new()
            }
        );

        let a = parse("me(true, false)").unwrap();
        assert_eq!(
            a,
            Expression::Call {
                lhs: Box::new(Expression::Ident("me".into())),
                arguments: vec![Expression::Bool(true), Expression::Bool(false)]
            }
        );
    }
}
