use chumsky::{input::ValueInput, prelude::*};

use crate::parser::{ast::Expression, lexer::Token};

use super::{
    ast::{Span, Spanned},
    operator::Operator,
};

#[allow(clippy::too_many_lines)]
#[must_use]
fn expr_parser<'src, I>(
) -> impl Parser<'src, I, Spanned<Expression>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    recursive(|expr| {
        // number and string literals
        let literal = select! {
            Token::Bool(b) => Expression::Bool(b.parse().unwrap()),
            Token::Number(n)=> Expression::Number(n.parse().unwrap()),
            Token::Identifier(name) => Expression::Ident(name.into()),
            Token::String(name) => Expression::String(name.into())
        }
        .labelled("value");

        let ident = select! { Token::Identifier(ident) => ident }.labelled("identifier");

        let items = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_leading()
            .allow_trailing()
            .collect::<Vec<_>>();

        let list = items
            .clone()
            .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
            .map(Expression::Array);

        let property_access = {
            expr.clone().foldl_with(
                just(Token::Dot).then(ident).repeated(),
                |lhs, (_, rhs), e| {
                    (
                        Expression::PropertyAccess {
                            lhs: Box::new(lhs),
                            property: rhs.into(),
                        },
                        e.span(),
                    )
                },
            )
        }
        .boxed();

        let atom = literal
            // identifier (variable)
            .or(ident.map(|x| Expression::Ident(x.into())))
            // list(s)
            .or(list)
            .map_with(|expr, e| (expr, e.span()))
            // normal expr but surrounded by parens
            .or(expr
                .clone()
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed)))
            .or(property_access)
            // Attempt to recover anything that looks like a parenthesised expression but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::ParenOpen,
                Token::ParenClosed,
                [
                    (Token::BracketOpen, Token::BracketClose),
                    (Token::CurlyBraceOpen, Token::CurlyBraceClose),
                ],
                |span| (Expression::Error, span),
            )))
            // Attempt to recover anything that looks like a list but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::BracketOpen,
                Token::BracketClose,
                [
                    (Token::ParenOpen, Token::ParenClosed),
                    (Token::CurlyBraceOpen, Token::CurlyBraceClose),
                ],
                |span| (Expression::Error, span),
            )))
            .boxed();

        let call = atom.foldl_with(
            items
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed))
                .repeated(),
            |func, arguments, e| {
                (
                    Expression::Call {
                        function: Box::new(func),
                        arguments,
                    },
                    e.span(),
                )
            },
        );

        let op = |operator: Operator| select! { Token::Operator(op) if op == operator => op };

        let unary = op(Operator::Sub)
            .or(op(Operator::Not))
            .repeated()
            .foldr_with(call, |op, rhs, e| {
                (
                    Expression::UnaryOp {
                        operator: op,
                        rhs: Box::new(rhs),
                    },
                    e.span(),
                )
            })
            .boxed();

        // macro for making binary rules because
        // writing out the types is just not worth it
        macro_rules! binary {
            ($atom: expr, $op: expr) => {{
                let op = $op;

                $atom
                    .clone()
                    .foldl_with(op.then($atom).repeated(), |lhs, (op, rhs), e| {
                        (
                            Expression::BinaryOp {
                                operator: op,
                                lhs: Box::new(lhs),
                                rhs: Box::new(rhs),
                            },
                            e.span(),
                        )
                    })
                    .boxed()
            }};
        }

        let product = binary!(
            unary,
            op(Operator::Mul)
                .or(op(Operator::Div))
                .or(op(Operator::Mod))
        );
        let sum = binary!(product, op(Operator::Add).or(op(Operator::Sub)));
        let xor = binary!(sum, op(Operator::Xor));
        let comparisons = binary!(
            xor,
            op(Operator::Equals)
                .or(op(Operator::NotEqual))
                .or(op(Operator::Greater))
                .or(op(Operator::GreaterOrEqual))
                .or(op(Operator::Less))
                .or(op(Operator::LessOrEqual))
        );
        let and = binary!(comparisons, op(Operator::And).or(op(Operator::Nor)));
        let or = binary!(and, op(Operator::Or));
        let assignment = binary!(or, op(Operator::Assign));

        let inline_expr = assignment.labelled("expression").as_context();

        inline_expr
    })
}

#[cfg(test)]
mod tests {

    mod expr {
        use chumsky::{input::Stream, prelude::*};
        use logos::Logos;

        use crate::parser::ast::Spanned;
        use crate::parser::{
            ast::Expression, lexer::Token, operator::Operator, parser::expr_parser,
        };

        fn parse(source: &str) -> Result<Spanned<Expression>, Vec<Rich<'_, Token<'_>>>> {
            let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
                Ok(tok) => (tok, span.into()),
                Err(()) => (Token::Error, span.into()),
            });

            let token_stream = Stream::from_iter(token_iter)
                // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
                // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
                .map((0..source.len()).into(), |(t, s): (_, _)| (t, s));

            expr_parser().parse(token_stream).into_result()
        }

        #[test]
        fn simple_expr() {
            let a = parse("1 + 2").unwrap();

            assert_eq!(
                a,
                (
                    Expression::BinaryOp {
                        operator: Operator::Add,
                        lhs: Box::new((Expression::Number(1.), SimpleSpan::new(0, 1))),
                        rhs: Box::new((Expression::Number(2.), SimpleSpan::new(4, 5)))
                    },
                    SimpleSpan::new(0, 5)
                )
            );
        }

        #[test]
        fn call_expr() {
            let a = parse("me").unwrap();

            assert_eq!(a.0, Expression::Ident("me".into()));

            let a = parse("me(true, false)").unwrap();
            assert_eq!(
                a.0,
                Expression::Call {
                    function: Box::new((Expression::Ident("me".into()), SimpleSpan::new(0, 2))),
                    arguments: vec![
                        (Expression::Bool(true), SimpleSpan::new(3, 7)),
                        (Expression::Bool(false), SimpleSpan::new(9, 14))
                    ]
                }
            );

            // let a = parse("x = me(true, false)").unwrap();
            // assert_eq!(
            //     a.0,
            //     Expression::Call {
            //         function: Box::new((Expression::Ident("me".into()), SimpleSpan::new(0, 2))),
            //         arguments: vec![]
            //     }
            // );
        }
    }
}
