use logos::Logos;

use crate::operator::Operator;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"([ \t\n\f]+)|(\/\/.*\n)|(\/\*.*\*\/)")]
pub enum Token {
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

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

    #[token("-", callback = |_| Operator::Sub)]
    #[token("+", callback = |_| Operator::Add)]
    #[token("*", callback = |_| Operator::Mul)]
    #[token("/", callback = |_| Operator::Div)]
    #[token("%", callback = |_| Operator::Mod)]
    #[token("=", callback = |_| Operator::Assign)]
    #[token("not", callback = |_| Operator::Not)]
    #[token("or", callback = |_| Operator::Or)]
    #[token("and", callback = |_| Operator::And)]
    #[token("nor", callback = |_| Operator::Nor)]
    #[token("xor", callback = |_| Operator::Xor)]
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

    #[token("let", priority = 100)]
    Let,

    #[token("var", priority = 100)]
    Var,

    #[token("func", priority = 100)]
    Func,

    #[token("entrypoint", priority = 100)]
    Entrypoint,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    #[regex(
        r#"@[^0-9\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"][^\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"]+"#, 
        callback = |lex|lex.slice()[1..].to_owned())]
    Directive(String),

    #[regex(
        r#"[^0-9\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"][^\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"]+"#, 
        priority = 0,
        callback = |lex| lex.slice().to_owned()
    )]
    Identifier(String),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| lex.slice().to_owned())]
    String(String),
}

#[cfg(test)]
mod tests {
    use logos::Logos;

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
}
