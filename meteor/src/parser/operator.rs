use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Operator {
    Sub,
    Add,
    Mul,
    Div,
    Mod,
    Assign,
    Not,
    Or,
    And,
    Nor,
    Xor,
    Equals,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

impl Operator {
    #[must_use]
    pub fn is_unary(&self) -> bool {
        matches!(self, Operator::Not | Operator::Sub)
    }

    #[must_use]
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Operator::Sub
                | Operator::Add
                | Operator::Mul
                | Operator::Div
                | Operator::Mod
                | Operator::Or
                | Operator::And
                | Operator::Assign
                | Operator::Nor
                | Operator::Xor
        )
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Operator::Sub => "-",
            Operator::Add => "+",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Mod => "mod",
            Operator::Assign => "=",
            Operator::Not => "not",
            Operator::Or => "or",
            Operator::And => "and",
            Operator::Nor => "nor",
            Operator::Xor => "xor",
            Operator::Equals => "==",
            Operator::NotEqual => "!=",
            Operator::Greater => ">",
            Operator::GreaterOrEqual => ">=",
            Operator::Less => "<",
            Operator::LessOrEqual => "<=",
        })
    }
}
