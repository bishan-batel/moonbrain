use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Sub,
    Add,
    Mul,
    Div,
    Mod,
    Not,
    Or,
    And,
    Assign,
    Nor,
    Xor,
}
