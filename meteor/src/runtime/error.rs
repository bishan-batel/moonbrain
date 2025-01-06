use std::io::Write;

use ariadne::{Cache, Label, Report};
use displaydoc::Display;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::parser::{
    ast::{self, Expression},
    operator::Operator,
    span::{Span as MSpan, Spanned},
    src::SourceId,
    symbol::Identifier,
};

use super::value::{Type, Value};

#[derive(Debug, Display, Error)]
pub enum RuntimeError {
    /// Failed to find variable of name {name}
    UnknownVariable {
        name: Identifier,
        expr: Spanned<Expression>,
    },

    /// Unknown type {data_type:?}
    UnknownType { data_type: Spanned<ast::Type> },

    /// Global 'main' is not a function
    InvalidMainFunc,

    /// Mismatch type to find variable of name {name}
    MismatchType {
        name: Identifier,
        data_type: Type,
        expr: Spanned<Expression>,
    },

    /// Unsupported operation in expression
    UnsupportedOperation(Spanned<Expression>),

    /// Unsupported operation in expression
    UnsupportedUnaryOperation(Operator, Spanned<Expression>, Value),

    /// Invalid property access
    InvalidPropertyAccess {
        obj: Spanned<Expression>,
        property: Identifier,
    },

    /// Array index out of bounds
    ArrayOutOfBounds {
        array: Spanned<Expression>,
        index: Spanned<Expression>,
    },

    /// Invalid property access
    CannotIndexIntoType {
        array: Spanned<Expression>,
        data_type: Type,
    },
}

impl RuntimeError {
    pub fn reason(&self) -> String {
        format!("{self}")
    }

    pub fn span(&self) -> &MSpan {
        lazy_static! {
            pub static ref EMPTY_SPAN: MSpan = MSpan::empty();
        }

        match self {
            RuntimeError::UnknownVariable {
                expr: (_, span), ..
            } => span,
            RuntimeError::UnknownType {
                data_type: (_, span),
            } => span,
            RuntimeError::MismatchType {
                expr: (_, span), ..
            } => span,
            RuntimeError::UnsupportedOperation((_, span)) => span,
            RuntimeError::UnsupportedUnaryOperation(_, (_, span), _) => span,
            RuntimeError::InvalidPropertyAccess { obj: (_, span), .. } => span,
            RuntimeError::ArrayOutOfBounds {
                index: (_, span), ..
            } => span,
            RuntimeError::CannotIndexIntoType {
                array: (_, span), ..
            } => span,
            RuntimeError::InvalidMainFunc => &EMPTY_SPAN,
        }
    }

    pub fn write(&self, cache: impl Cache<SourceId>, writer: impl Write) {
        let span: &MSpan = self.span();
        Report::build(ariadne::ReportKind::Error, span.clone())
            .with_code(3)
            .with_message("Error")
            .with_label(
                Label::new(span.clone())
                    .with_message(format!("{self}"))
                    .with_color(ariadne::Color::Red),
            )
            .finish()
            .write(cache, writer)
            .unwrap();
    }
}
