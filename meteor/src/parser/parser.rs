use chumsky::{input::ValueInput, prelude::*};

use crate::{
    parser::{
        ast::{Expression, Function, Type, VariableMeta},
        lexer::Token,
        symbol::Identifier,
    },
    runtime::memory::Mutability,
};

use super::{
    ast::{Directive, Program},
    operator::Operator,
    span::{Span, Spanned},
};

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn program_parser<'src, I>(
) -> impl Parser<'src, I, Program, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    let expr = expr_parser::<'src, I>();

    let directive = select! {
        Token::Directive(dir) => Directive::new(dir, vec![])
    }
    .map_with(|x, e| (x, e.span()));

    let directives = directive.repeated().collect::<Vec<_>>();

    let expressions = expr
        .then_ignore(just(Token::Semicolon).or_not())
        .repeated()
        .collect::<Vec<_>>();

    directives
        .then(expressions)
        .map_with(|(directives, exprs), _| Program::new(directives, exprs))
}

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
            Token::String(name) => Expression::String(name.into()),
            Token::Nil => Expression::Nil
        }
        .labelled("value");

        let ident =
            select! { Token::Identifier(ident) => Identifier::from(ident) }.labelled("identifier");

        let dict = (ident
            .clone()
            .then_ignore(just(Token::Operator(Operator::Assign)))
            .then(expr.clone()))
        .separated_by(just(Token::Comma))
        .allow_leading()
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::CurlyBraceOpen), just(Token::CurlyBraceClose))
        .map_with(|pairs, e| (Expression::Dictionary(pairs), e.span()));

        let r#type = ident
            .clone()
            .map_with(|ident, e| (Type::Named(ident), e.span()));

        let variable_declare = ident
            .clone()
            .then(just(Token::Colon).ignore_then(r#type).or_not())
            .map_with(|(name, r#type), e| {
                (
                    VariableMeta::new(name, r#type, Mutability::Mutable),
                    e.span(),
                )
            })
            .boxed();

        let items = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_leading()
            .allow_trailing()
            .collect::<Vec<_>>();

        let block = expr
            .clone()
            .then_ignore(just(Token::Semicolon).or_not())
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::CurlyBraceOpen), just(Token::CurlyBraceClose))
            .map_with(|expressions, e| (Expression::Block(expressions), e.span()))
            .boxed();

        let lambda = just(Token::Func)
            .ignore_then(ident.clone().or_not())
            .then(
                // single/no argument like func x => x*2
                variable_declare
                    .clone()
                    .map(|arg| vec![arg])
                    // or args in parenthesis like func(a, b) => a+b
                    .or(variable_declare
                        .clone()
                        .separated_by(just(Token::Comma))
                        .allow_leading()
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed)))
                    .or_not(),
            )
            .then(
                block
                    .clone()
                    .or(just(Token::FatArrow).ignore_then(expr.clone())),
            )
            .map_with(|((name, args), expr), e| {
                let func =
                    Expression::Func(Box::new(Function::new(args.unwrap_or_default(), expr)));
                if let Some(name) = name {
                    (
                        Expression::Let {
                            meta: (
                                VariableMeta::new(name, None, Mutability::Constant),
                                e.span(),
                            ),
                            init: Box::new((func, e.span())),
                        },
                        e.span(),
                    )
                } else {
                    (func, e.span())
                }
            })
            .boxed();

        let let_expr = just(Token::Let)
            .ignore_then(variable_declare.clone())
            .then_ignore(just(Token::Operator(Operator::Assign)))
            .then(expr.clone())
            .map_with(|(meta, init), e| {
                (
                    Expression::Let {
                        meta,
                        init: Box::new(init),
                    },
                    e.span(),
                )
            })
            .boxed();

        let list = items
            .clone()
            .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
            .map(Expression::Array);

        let atom = literal
            // identifier (variable)
            .or(ident.map(|x| Expression::Ident(x)))
            // list(s)
            .or(list)
            .map_with(|expr, e| (expr, e.span()))
            .or(lambda)
            .or(let_expr)
            .or(dict)
            .or(just(Token::Do).ignore_then(block.clone()))
            // normal expr but surrounded by parens
            .or(expr
                .clone()
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClosed)))
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

        let property_access = {
            atom.foldl_with(
                just(Token::Dot).ignore_then(ident).repeated(),
                |lhs, property, e| {
                    (
                        Expression::PropertyAccess {
                            lhs: Box::new(lhs),
                            property,
                        },
                        e.span(),
                    )
                },
            )
        }
        .boxed();

        let call = property_access
            .foldl_with(
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
            )
            .boxed();

        let array_index = call
            .foldl_with(
                expr.clone()
                    .delimited_by(just(Token::BracketOpen), just(Token::BracketClose))
                    .repeated(),
                |lhs, idx, e| {
                    (
                        Expression::ArrayIndex {
                            lhs: Box::new(lhs),
                            index: Box::new(idx),
                        },
                        e.span(),
                    )
                },
            )
            .boxed()
            .labelled("array index")
            .as_context();

        let op = |operator: Operator| select! { Token::Operator(op) if op == operator => op };

        let unary = op(Operator::Sub)
            .or(op(Operator::Not))
            .repeated()
            .foldr_with(array_index, |op, rhs, e| {
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

        let assignment = binary!(or, op(Operator::Assign)).labelled("assignment");

        let inline_expr = assignment.labelled("expression").as_context();

        let while_expr = just(Token::While)
            .or(just(Token::Unless))
            .then(expr.clone())
            .then(block.clone())
            .map_with(|((unless, cond), body), e| {
                let cond = if matches!(unless, Token::Unless) {
                    let span = cond.1.clone();
                    (
                        Expression::UnaryOp {
                            operator: Operator::Not,
                            rhs: Box::new(cond),
                        },
                        span,
                    )
                } else {
                    cond
                };

                (
                    Expression::While {
                        condition: Box::new(cond),
                        then: Box::new(body),
                    },
                    e.span(),
                )
            })
            .boxed();

        let if_expr = recursive(|if_expr| {
            just(Token::If)
                .ignore_then(expr.clone())
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(block.clone().or(if_expr))
                        .or_not(),
                )
                .map_with(|((cond, body), or_else), e| {
                    (
                        Expression::If {
                            condition: Box::new(cond),
                            then: Box::new(body),
                            or_else: Box::new(or_else.unwrap_or((Expression::Nil, e.span()))),
                        },
                        e.span(),
                    )
                })
                .boxed()
        });

        inline_expr.or(while_expr).or(if_expr)
    })
}

#[cfg(test)]
mod tests {

    mod expr {
        use chumsky::{input::Stream, prelude::*};
        use logos::Logos;

        use crate::parser::span::{Span, Spanned};
        use crate::parser::{
            ast::Expression, lexer::Token, operator::Operator, parser::expr_parser,
        };

        fn parse(source: &str) -> Result<Spanned<Expression>, Vec<Rich<'_, Token<'_>, Span>>> {
            let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
                Ok(tok) => (tok, Span::empty()),
                Err(()) => (Token::Error, Span::empty()),
            });

            let token_stream = Stream::from_iter(token_iter)
                // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
                // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
                .map(Span::empty(), |(t, s): (_, _)| (t, s));

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
                        lhs: Box::new((Expression::Number(1.), Span::empty())),
                        rhs: Box::new((Expression::Number(2.), Span::empty()))
                    },
                    Span::empty()
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
                    function: Box::new((Expression::Ident("me".into()), Span::empty())),
                    arguments: vec![
                        (Expression::Bool(true), Span::empty()),
                        (Expression::Bool(false), Span::empty())
                    ]
                }
            );

            let a = parse("x = me(true, false)").unwrap();
            assert_eq!(
                a.0,
                Expression::BinaryOp {
                    lhs: Box::new((Expression::Ident("x".into()), Span::empty())),
                    operator: Operator::Assign,
                    rhs: Box::new((
                        Expression::Call {
                            function: Box::new((Expression::Ident("me".into()), Span::empty())),
                            arguments: vec![
                                (Expression::Bool(true), Span::empty()),
                                (Expression::Bool(false), Span::empty())
                            ]
                        },
                        Span::empty()
                    ))
                }
            );
        }

        #[test]
        fn call_expr_noarg() {
            let a = parse("me()").unwrap();
            assert_eq!(
                a.0,
                Expression::Call {
                    function: Box::new((Expression::Ident("me".into()), Span::empty())),
                    arguments: vec![]
                }
            );
        }

        #[test]
        fn array_index() {
            let a = parse("me[10]").unwrap();
            assert_eq!(
                a.0,
                Expression::ArrayIndex {
                    lhs: Box::new((Expression::Ident("me".into()), Span::empty())),
                    index: Box::new((Expression::Number(10.), Span::empty())),
                }
            );
        }
    }

    mod prog {
        use chumsky::{input::Stream, prelude::*};
        use logos::Logos;

        use crate::parser::ast::{Directive, Program};
        use crate::parser::parser::program_parser;
        use crate::parser::span::{Span, Spanned};
        use crate::parser::{
            ast::Expression, lexer::Token, operator::Operator, parser::expr_parser,
        };
        use indoc::indoc;

        fn parse(source: &str) -> Result<Program, Vec<Rich<'_, Token<'_>, Span>>> {
            let token_iter = Token::lexer(source).spanned().map(|(tok, span)| match tok {
                Ok(tok) => (tok, Span::empty()),
                Err(()) => (Token::Error, Span::empty()),
            });

            let token_stream = Stream::from_iter(token_iter)
                // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
                // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
                .map(Span::empty(), |(t, s): (_, _)| (t, s));

            program_parser().parse(token_stream).into_result()
        }

        #[test]
        fn prog_simple() {
            assert_eq!(
                parse(r#"@x"#).unwrap(),
                Program::new(vec![(Directive::new("x", vec![]), Span::empty())], vec![])
            );

            assert_eq!(
                parse(r#"@x @bruh"#).unwrap(),
                Program::new(
                    vec![
                        (Directive::new("x", vec![]), Span::empty()),
                        (Directive::new("bruh", vec![]), Span::empty()),
                    ],
                    vec![]
                )
            );

            assert_eq!(
                parse(indoc! {r#"
                    @x
                    @bruh 
                "#})
                .unwrap(),
                Program::new(
                    vec![
                        (Directive::new("x", vec![]), Span::empty()),
                        (Directive::new("bruh", vec![]), Span::empty()),
                    ],
                    vec![]
                )
            );
        }
    }
}
