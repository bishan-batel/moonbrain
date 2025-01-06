use std::fmt::Display;

use logos::Logos;

use crate::parser::operator::Operator;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"([ \t\n\f]+)|(\/\/.*\n)|(\/\*.*\*\/)")]
pub enum Token<'a> {
    #[token("false")]
    #[token("true")]
    Bool(&'a str),

    #[token("{")]
    CurlyBraceOpen,

    #[token("}")]
    CurlyBraceClose,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClosed,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token(";")]
    Semicolon,

    #[token("=>")]
    FatArrow,

    #[token("-",   |_| Operator::Sub)]
    #[token("+",   |_| Operator::Add)]
    #[token("*",   |_| Operator::Mul)]
    #[token("/",   |_| Operator::Div)]
    #[token("%",   |_| Operator::Mod)]
    #[token("mod", |_| Operator::Mod)]
    #[token("=",   |_| Operator::Assign)]
    #[token("not", |_| Operator::Not)]
    #[token("or",  |_| Operator::Or)]
    #[token("and", |_| Operator::And)]
    #[token("nor", |_| Operator::Nor)]
    #[token("xor", |_| Operator::Xor)]
    #[token("==",  |_| Operator::Equals)]
    #[token("!=",  |_| Operator::NotEqual)]
    #[token(">",   |_| Operator::Greater)]
    #[token(">=",  |_| Operator::GreaterOrEqual)]
    #[token("<",   |_| Operator::Less)]
    #[token("<=",  |_| Operator::LessOrEqual)]
    Operator(Operator),

    #[token("nil", priority = 100)]
    Nil,

    #[token("return", priority = 100)]
    Return,

    #[token("if", priority = 100)]
    If,

    #[token("unless", priority = 100)]
    Unless,

    #[token("else", priority = 100)]
    Else,

    #[token("while", priority = 100)]
    While,

    #[token("until", priority = 100)]
    Until,

    #[token("for", priority = 100)]
    For,

    #[token("in", priority = 100)]
    In,

    #[token("do", priority = 100)]
    Do,

    #[token("let", priority = 100)]
    Let,

    #[token("var", priority = 100)]
    Var,

    #[token("func", priority = 100)]
    #[token("fn", priority = 100)]
    #[token("function", priority = 100)]
    #[token("fun)", priority = 100)]
    Func,

    #[token("entrypoint", priority = 100)]
    Entrypoint,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    Number(&'a str),

    #[regex(
        r#"@[^0-9\s\=\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"][^\s\=\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"]*"#, 
        callback = |lex| &lex.slice()[1..])]
    Directive(&'a str),

    #[regex(
        r#"[^0-9\s\=\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"][^\s\=\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"]*"#, 
        priority = 0
    )]
    Identifier(&'a str),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| &lex.slice()[1..(lex.slice().len() - 1)])]
    String(&'a str),

    Error,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Bool(state) => f.write_str(state),
            Token::CurlyBraceOpen => f.write_str("{"),
            Token::CurlyBraceClose => f.write_str("}"),
            Token::BracketOpen => f.write_str("["),
            Token::BracketClose => f.write_str("]"),
            Token::ParenOpen => f.write_str("("),
            Token::ParenClosed => f.write_str(")"),
            Token::Colon => f.write_str(":"),
            Token::Comma => f.write_str(","),
            Token::Dot => f.write_str("."),
            Token::Semicolon => f.write_str(";"),
            Token::FatArrow => f.write_str("=>"),
            Token::Operator(operator) => f.write_fmt(format_args!("{operator}")),
            Token::Nil => f.write_str("nil"),
            Token::Return => f.write_str("return"),
            Token::If => f.write_str("if"),
            Token::Unless => f.write_str("unless"),
            Token::Else => f.write_str("else"),
            Token::While => f.write_str("while"),
            Token::Until => f.write_str("until"),
            Token::For => f.write_str("for"),
            Token::In => f.write_str("in"),
            Token::Let => f.write_str("let"),
            Token::Var => f.write_str("var"),
            Token::Func => f.write_str("func"),
            Token::Entrypoint => f.write_str("entrypoint"),
            Token::Number(num) => f.write_fmt(format_args!("{num}")),
            Token::Directive(dir) => f.write_fmt(format_args!("@{dir}")),
            Token::Identifier(ident) => f.write_fmt(format_args!("{ident}")),
            Token::String(str) => f.write_fmt(format_args!("{str:#?}")),
            Token::Error => f.write_str("[ERROR]"),
            Token::Do => f.write_str("do"),
        }
    }
}

#[cfg(test)]
mod tests {

    use logos::Logos;

    use crate::parser::operator::Operator;

    use super::Token;

    #[test]
    fn nil() {
        let mut lexer = Token::lexer("nil");
        assert_eq!(lexer.next(), Some(Ok(Token::Nil)));

        let mut lexer = Token::lexer("nil nil nil");
        assert_eq!(lexer.next(), Some(Ok(Token::Nil)));
        assert_eq!(lexer.next(), Some(Ok(Token::Nil)));
        assert_eq!(lexer.next(), Some(Ok(Token::Nil)));
    }

    #[test]
    fn whitespace() {
        let mut lexer = Token::lexer("  \n  \t\t");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn comments() {
        let mut lexer = Token::lexer("// bruh moment\n\n");
        assert_eq!(lexer.next(), None);

        let mut lexer = Token::lexer("// bruh moment\n\nnil // hello world \nnil");
        assert_eq!(lexer.next(), Some(Ok(Token::Nil)));
        assert_eq!(lexer.next(), Some(Ok(Token::Nil)));
    }

    #[test]
    fn general() {
        color_eyre::install().unwrap();

        let toks: Vec<_> = Token::lexer(
            r#"
            if true {
                a = (a + 1)
            } else {
                b.what = "huh"
            }
        "#,
        )
        .collect();

        assert_eq!(
            toks,
            vec![
                Ok(Token::If),
                Ok(Token::Bool("true")),
                Ok(Token::CurlyBraceOpen),
                Ok(Token::Identifier("a")),
                Ok(Token::Operator(Operator::Assign)),
                Ok(Token::ParenOpen),
                Ok(Token::Identifier("a")),
                Ok(Token::Operator(Operator::Add)),
                Ok(Token::Number("1")),
                Ok(Token::ParenClosed),
                Ok(Token::CurlyBraceClose),
                Ok(Token::Else),
                Ok(Token::CurlyBraceOpen),
                Ok(Token::Identifier("b")),
                Ok(Token::Dot),
                Ok(Token::Identifier("what")),
                Ok(Token::Operator(Operator::Assign)),
                Ok(Token::String("huh")),
                Ok(Token::CurlyBraceClose),
            ]
        );
    }
}
