use serde::Serialize;

use crate::{
    parser::{operator::Operator, symbol::Identifier},
    runtime::memory::Mutability,
};

use super::span::Spanned;

/// AST Representation of a typical program (*one file*)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    directives: Vec<Spanned<Directive>>,
    expressions: Vec<Spanned<Expression>>,
}

/// AST Representation of a Directive
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Directive {
    name: Identifier,
    params: Vec<Spanned<Expression>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Type {
    Named(Identifier),
    Generic(Box<Type>, Vec<Identifier>),
}

/// AST Representation of a variable declaration,
/// with a name and optional type (unevaluated, so just a identifier binding if provided)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VariableMeta {
    name: Identifier,
    data_type: Option<Spanned<Type>>,
    mutability: Mutability,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Function {
    arguments: Vec<Spanned<VariableMeta>>,
    body: Spanned<Expression>,
}

/// AST Expression
#[derive(Debug, Clone, PartialEq, Serialize)]
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

    Dictionary(Vec<(Identifier, Spanned<Expression>)>),

    Func(Box<Function>),

    Let {
        meta: Spanned<VariableMeta>,
        init: Box<Spanned<Self>>,
    },

    Block(Vec<Spanned<Self>>),

    If {
        condition: Box<Spanned<Self>>,
        then: Box<Spanned<Self>>,
        or_else: Box<Spanned<Self>>,
    },

    While {
        condition: Box<Spanned<Self>>,
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

impl Function {
    pub fn new(arguments: Vec<Spanned<VariableMeta>>, body: Spanned<Expression>) -> Self {
        Self { arguments, body }
    }

    pub fn arguments(&self) -> &[Spanned<VariableMeta>] {
        &self.arguments
    }

    pub fn body(&self) -> &Spanned<Expression> {
        &self.body
    }
}

impl Program {
    #[must_use]
    pub fn new(directives: Vec<Spanned<Directive>>, expressions: Vec<Spanned<Expression>>) -> Self {
        Self {
            directives,
            expressions,
        }
    }

    #[must_use]
    pub fn directives(&self) -> &Vec<Spanned<Directive>> {
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
    pub fn new(name: Identifier, data_type: Option<Spanned<Type>>, mutability: Mutability) -> Self {
        Self {
            name,
            data_type,
            mutability,
        }
    }

    #[must_use]
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    #[must_use]
    pub fn data_type(&self) -> Option<&Spanned<Type>> {
        self.data_type.as_ref()
    }

    pub fn mutablity(&self) -> Mutability {
        self.mutability
    }
}
