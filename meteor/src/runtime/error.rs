use thiserror::Error;

use crate::parser::{
    ast::{self, Expression, Spanned},
    operator::Operator,
    symbol::Identifier,
};

use super::value::{Type, Value};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Failed to find variable of name {name}")]
    UnknownVariable {
        name: Identifier,
        expr: Spanned<Expression>,
    },

    #[error("Unknown type {:?}", data_type.0)]
    UnknownType { data_type: Spanned<ast::Type> },

    #[error("Failed to find variable of name {name}")]
    MismatchType {
        name: Identifier,
        data_type: Type,
        expr: Spanned<Expression>,
    },

    #[error("Unsupported operation in expression")]
    UnsupportedOperation(Spanned<Expression>),

    #[error("Unsupported operation in expression")]
    UnsupportedUnaryOperation(Operator, Spanned<Expression>, Value),

    #[error("Invalid property access")]
    InvalidPropertyAccess {
        obj: Spanned<Expression>,
        property: Identifier,
    },

    #[error("Invalid property access")]
    ArrayOutOfBounds {
        array: Spanned<Expression>,
        index: Spanned<Expression>,
    },

    #[error("Invalid property access")]
    CannotIndexIntoType {
        array: Spanned<Expression>,
        data_type: Type,
    },
}
