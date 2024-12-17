use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
