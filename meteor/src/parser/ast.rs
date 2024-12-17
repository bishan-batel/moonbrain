use crate::parser::{operator::Operator, symbol::Identifier};

#[derive(Debug)]
pub struct Program {
    directives: Vec<Directive>,
    expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct Directive {
    name: Identifier,
    params: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariableMeta {
    name: Identifier,
    data_type: Option<Identifier>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Ident(Identifier),
    String(String),
    Bool(bool),
    Number(f64),

    Fn {
        name: Identifier,
        body: Identifier,
    },

    Let {
        meta: VariableMeta,
        initial: Box<Expression>,
    },

    BinaryOp {
        operator: Operator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },

    UnaryOp {
        operator: Operator,
        rhs: Box<Expression>,
    },

    Call {
        lhs: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

impl Program {
    #[must_use]
    pub fn new(directives: Vec<Directive>, expressions: Vec<Expression>) -> Self {
        Self {
            directives,
            expressions,
        }
    }

    #[must_use]
    pub fn directives(&self) -> &Vec<Directive> {
        &self.directives
    }

    pub fn expressions(&self) -> &Vec<Expression> {
        &self.expressions
    }
}

impl Directive {
    #[must_use]
    pub fn new(name: Identifier, params: Vec<Expression>) -> Self {
        Self { name, params }
    }

    #[must_use]
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    #[must_use]
    pub fn params(&self) -> &Vec<Expression> {
        &self.params
    }
}

impl VariableMeta {
    #[must_use]
    fn new(name: Identifier, data_type: Option<Identifier>) -> Self {
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
