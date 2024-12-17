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

    #[token("-",   |_| Operator::Sub)]
    #[token("+",   |_| Operator::Add)]
    #[token("*",   |_| Operator::Mul)]
    #[token("/",   |_| Operator::Div)]
    #[token("%",   |_| Operator::Mod)]
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

    #[token("let", priority = 100)]
    Let,

    #[token("var", priority = 100)]
    Var,

    #[token("func", priority = 100)]
    Func,

    #[token("entrypoint", priority = 100)]
    Entrypoint,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    Number(&'a str),

    #[regex(
        r#"@[^0-9\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"][^\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"]+"#, 
        callback = |lex| &lex.slice()[1..])]
    Directive(&'a str),

    #[regex(
        r#"[^0-9\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"][^\s\[\]\(\)\!\@\#\$\%\^\&\*\-\+\{\}<>\,\.\`\~\/\\\;\:\'\"]+"#, 
        priority = 0
    )]
    Identifier(&'a str),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    String(&'a str),

    Error,
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
