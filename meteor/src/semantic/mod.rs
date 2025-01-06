use core::fmt;
use std::collections::{HashMap, HashSet};

use chumsky::container::{Container, Seq};
use displaydoc::Display;

use crate::parser::{
    ast::{Expression, Program},
    span::Spanned,
    symbol::Identifier,
};

#[derive(Display, Debug, Clone, PartialEq)]
pub enum DiagnosticKind {
    /// Unknown variable `{0}`
    UnknownVariable(Identifier),

    /// Invalid top level expression, can only be a variable or function declaration
    InvalidTopLevel,

    /// Infinite loops are not supported and will crash your chip
    InfiniteLoop,

    /// Condition is always {0}
    ConditionIsConstant(bool),

    /// Arrays cannot have negative indexes
    NegativeArrayIndex,

    /// Arrays cannot have a non integer index
    FractionalArrayIndex,

    /// Multiple arguments named `{0}`
    DuplicateArgumentName(Identifier),
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Hint
    Hint,

    /// Warning
    Warning,

    /// Error
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic<'a> {
    pub kind: DiagnosticKind,
    pub severity: Severity,
    pub span: &'a Spanned<Expression>,
}

impl Diagnostic<'_> {
    pub fn reason(&self) -> String {
        format!("{}", self.kind)
    }
}

struct Analyzer<'a> {
    program: &'a Program,
    diagnoses: Vec<Diagnostic<'a>>,
}

impl<'a> Analyzer<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            diagnoses: vec![],
            program,
        }
    }

    fn analyze_prog(mut self) -> Vec<Diagnostic<'a>> {
        self.validate_top_level();

        for expr in self.program.expressions() {
            self.analyze(expr);
        }

        self.diagnoses
    }

    fn validate_top_level(&mut self) {
        for prog in self.program.expressions() {
            match prog.0 {
                Expression::Func(..) | Expression::Let { .. } => {}

                _ => {
                    self.diagnose(Diagnostic {
                        kind: DiagnosticKind::InvalidTopLevel,
                        severity: Severity::Warning,
                        span: prog,
                    });
                }
            }
        }
    }

    fn analyze_each(&mut self, exprs: impl Iterator<Item = &'a Spanned<Expression>>) {
        for expr in exprs {
            self.analyze(expr)
        }
    }

    fn analyze(&mut self, expr: &'a Spanned<Expression>) {
        match &expr.0 {
            Expression::While { condition, then } => {
                self.check_condition(expr, condition);

                if let Expression::Bool(cond) = condition.0 {
                    if cond {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::InfiniteLoop,
                            severity: Severity::Warning,
                            span: expr,
                        });
                    }
                }

                self.analyze(condition);
                self.analyze(then);
            }

            Expression::Array(vec) => self.analyze_each(vec.iter()),

            Expression::Block(vec) => self.analyze_each(vec.iter()),

            Expression::Dictionary(vec) => self.analyze_each(vec.iter().map(|(_, v)| v)),

            Expression::Func(function) => {
                let mut duplicates = HashMap::new();

                for arg in function.arguments() {
                    let name = arg.0.name().clone();

                    if let Some(count) = duplicates.get_mut(&name) {
                        *count += 1;
                    } else {
                        duplicates.insert(name, 1usize);
                    }
                }

                for (name, _) in duplicates.into_iter().filter(|(_, c)| *c > 1) {
                    self.diagnose(Diagnostic {
                        kind: DiagnosticKind::DuplicateArgumentName(name),
                        severity: Severity::Warning,
                        span: expr,
                    });
                }

                self.analyze(function.body());
            }

            Expression::Let { meta: _, init } => self.analyze(init),

            Expression::If {
                condition,
                then,
                or_else,
            } => {
                self.check_condition(expr, condition);
                self.analyze(condition);
                self.analyze(then);
                self.analyze(or_else);
            }

            Expression::PropertyAccess { lhs, property: _ } => {
                self.analyze(lhs);
            }

            Expression::ArrayIndex { lhs, index } => {
                self.analyze(lhs);
                self.analyze(index);

                if let Expression::Number(n) = index.0 {
                    if n < 0. {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::NegativeArrayIndex,
                            severity: Severity::Warning,
                            span: index,
                        });
                    }

                    if n.trunc() != n {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::FractionalArrayIndex,
                            severity: Severity::Warning,
                            span: index,
                        });
                    }
                }
            }

            Expression::BinaryOp {
                lhs,
                operator: _,
                rhs,
            } => {
                self.analyze(lhs);
                self.analyze(rhs);
            }

            Expression::UnaryOp { operator: _, rhs } => {
                self.analyze(rhs);
            }

            Expression::Call {
                function,
                arguments,
            } => {
                self.analyze(function);
                self.analyze_each(arguments.iter());
            }

            Expression::Error
            | Expression::Nil
            | Expression::Ident(..)
            | Expression::String(_)
            | Expression::Bool(_)
            | Expression::Number(_) => {}
        }
    }

    fn diagnose(&mut self, diagnostic: Diagnostic<'a>) {
        self.diagnoses.push(diagnostic);
    }

    fn check_condition(&mut self, expr: &'a Spanned<Expression>, cond: &'a Spanned<Expression>) {
        let b = match cond.0 {
            Expression::Bool(b) => b,
            Expression::Number(n) => n != 0.,
            Expression::String(ref str) => str.is_empty(),
            Expression::Nil => false,
            _ => return,
        };

        self.diagnose(Diagnostic {
            kind: DiagnosticKind::ConditionIsConstant(b),
            severity: Severity::Hint,
            span: expr,
        });
    }
}

pub fn analyze<'a>(program: &'a Program) -> Vec<Diagnostic<'a>> {
    Analyzer::new(program).analyze_prog()
}
