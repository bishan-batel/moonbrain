use chumsky::span::SimpleSpan;

use crate::parser::{operator::Operator, symbol::Identifier};

pub type Span = SimpleSpan;

pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    directives: Vec<Directive>,
    expressions: Vec<Spanned<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    name: Identifier,
    params: Vec<Spanned<Expression>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableMeta {
    name: Identifier,
    data_type: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Error,
    Nil,
    Ident(Identifier),
    String(String),
    Bool(bool),
    Number(f64),

    Array(Vec<Spanned<Self>>),

    Fn {
        name: Identifier,
        body: Identifier,
    },

    Let {
        meta: VariableMeta,
        initial: Box<Spanned<Self>>,
    },

    BinaryOp {
        operator: Operator,
        lhs: Box<Spanned<Self>>,
        rhs: Box<Spanned<Self>>,
    },

    UnaryOp {
        operator: Operator,
        rhs: Box<Spanned<Self>>,
    },

    Call {
        lhs: Box<Spanned<Self>>,
        arguments: Vec<Spanned<Self>>,
    },
}

impl Program {
    #[must_use]
    pub fn new(directives: Vec<Directive>, expressions: Vec<Spanned<Expression>>) -> Self {
        Self {
            directives,
            expressions,
        }
    }

    #[must_use]
    pub fn directives(&self) -> &Vec<Directive> {
        &self.directives
    }

    pub fn expressions(&self) -> &Vec<Spanned<Expression>> {
        &self.expressions
    }
}

impl Directive {
    #[must_use]
    pub fn new(name: impl Into<Identifier>, params: Vec<Spanned<Expression>>) -> Self {
        let name = name.into();
        Self { name, params }
    }

    #[must_use]
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    #[must_use]
    pub fn params(&self) -> &Vec<Spanned<Expression>> {
        &self.params
    }
}

impl VariableMeta {
    #[must_use]
    fn new(name: impl Into<Identifier>, data_type: Option<Identifier>) -> Self {
        let name = name.into();
        Self { name, data_type }
    }

    #[must_use]
    pub fn typed(name: Identifier, data_type: Identifier) -> Self {
        Self::new(name, Some(data_type))
    }

    #[must_use]
    pub fn untyped(name: Identifier) -> Self {
        Self::new(name, None)
    }

    #[must_use]
    pub fn data_type(&self) -> Option<&Identifier> {
        self.data_type.as_ref()
    }

    #[must_use]
    pub fn name(&self) -> &Identifier {
        &self.name
    }
}
