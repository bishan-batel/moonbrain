use chumsky::span::SimpleSpan;

use crate::parser::{operator::Operator, symbol::Identifier};

/// Span information for a given token or AST Expression
pub type Span = SimpleSpan;

/// A type with associated span
pub type Spanned<T> = (T, Span);

/// AST Representation of a typical program (*one file*)
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    directives: Vec<Directive>,
    expressions: Vec<Spanned<Expression>>,
}

/// AST Representation of a Directive
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    name: Identifier,
    params: Vec<Spanned<Expression>>,
}

/// AST Representation of a variable declaration,
/// with a name and optional type (unevaluated, so just a identifier binding if provided)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableMeta {
    name: Identifier,
    data_type: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    arguments: Vec<VariableMeta>,
    body: Spanned<Expression>,
}

/// AST Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Invalid expression, do not evaluate this case.
    Error,

    /// Nil value literal
    Nil,

    /// Identifier binding
    Ident(Identifier),

    /// String Literal
    String(String),

    /// Boolean Literal
    Bool(bool),

    /// Number Literal
    Number(f64),

    /// Array Literal
    Array(Vec<Spanned<Self>>),

    Fn {
        name: Identifier,
        body: Identifier,
    },

    Let {
        meta: VariableMeta,
        initial: Box<Spanned<Self>>,
    },

    Block {
        expressions: Vec<Spanned<Self>>,
    },

    If {
        condition: Spanned<Box<Self>>,
        then: Box<Spanned<Self>>,
        or_else: Box<Spanned<Self>>,
    },

    While {
        condition: Spanned<Box<Self>>,
        then: Box<Spanned<Self>>,
    },

    PropertyAccess {
        lhs: Box<Spanned<Self>>,
        property: Identifier,
    },

    ArrayIndex {
        lhs: Box<Spanned<Self>>,
        index: Box<Spanned<Self>>,
    },

    BinaryOp {
        lhs: Box<Spanned<Self>>,
        operator: Operator,
        rhs: Box<Spanned<Self>>,
    },

    UnaryOp {
        operator: Operator,
        rhs: Box<Spanned<Self>>,
    },

    Call {
        function: Box<Spanned<Self>>,
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
