use core::fmt;
use std::collections::{HashMap, HashSet};

use chumsky::container::{Container, Seq};
use displaydoc::Display;

use crate::parser::{
    ast::{Expression, Program},
    operator::Operator,
    span::{Span, Spanned},
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

    /// Expression is being ignored
    IgnoredOperation,

    /// Empty block has no use
    EmptyBlock,

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
    pub span: &'a Span,
}

impl Diagnostic<'_> {
    pub fn reason(&self) -> String {
        format!("{}", self.kind)
    }
}

pub struct Diagnoses<'a> {
    pub diagnostics: Vec<Diagnostic<'a>>,
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

    fn analyze_prog(mut self) -> Diagnoses<'a> {
        self.validate_top_level();

        for expr in self.program.expressions() {
            self.analyze_inline(expr);
        }

        Diagnoses {
            diagnostics: self.diagnoses,
        }
    }

    fn validate_top_level(&mut self) {
        for prog in self.program.expressions() {
            match prog.0 {
                Expression::Func(..) | Expression::Let { .. } => {}

                _ => {
                    self.diagnose(Diagnostic {
                        kind: DiagnosticKind::InvalidTopLevel,
                        severity: Severity::Warning,
                        span: &prog.1,
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

    fn analyze_each_inline(&mut self, exprs: impl Iterator<Item = &'a Spanned<Expression>>) {
        for expr in exprs {
            self.analyze_inline(expr)
        }
    }

    fn analyze(&mut self, expr: &'a Spanned<Expression>) {
        match &expr.0 {
            Expression::Error
            | Expression::Let { .. }
            | Expression::If { .. }
            | Expression::While { .. }
            | Expression::Call { .. } => {}

            Expression::BinaryOp { operator, .. } if operator != &Operator::Assign => {}

            Expression::Block(exprs) => {
                if exprs.is_empty() {
                    self.diagnose(Diagnostic {
                        kind: DiagnosticKind::EmptyBlock,
                        severity: Severity::Warning,
                        span: &expr.1,
                    });
                }
            }

            _ => self.diagnose(Diagnostic {
                kind: DiagnosticKind::IgnoredOperation,
                severity: Severity::Warning,
                span: &expr.1,
            }),
        }
        self.analyze_inline(expr);
    }

    fn analyze_inline(&mut self, expr: &'a Spanned<Expression>) {
        match &expr.0 {
            Expression::While { condition, then } => {
                self.check_condition(expr, condition);

                if let Expression::Bool(cond) = condition.0 {
                    if cond {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::InfiniteLoop,
                            severity: Severity::Warning,
                            span: &expr.1,
                        });
                    }
                }

                self.analyze_inline(condition);
                self.analyze_inline(then);
            }

            Expression::Array(vec) => self.analyze_each_inline(vec.iter()),

            Expression::Block(vec) => {
                for i in 0..(vec.len() - 1) {
                    self.analyze(&vec[i]);
                }
                if let Some(last) = vec.last() {
                    self.analyze_inline(last);
                }
            }

            Expression::Dictionary(vec) => self.analyze_each_inline(vec.iter().map(|(_, v)| v)),

            Expression::Func(function) => {
                let mut duplicates = HashSet::new();

                for arg in function.arguments() {
                    let name = arg.0.name().clone();

                    if duplicates.contains(&name) {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::DuplicateArgumentName(name),
                            severity: Severity::Warning,
                            span: &arg.1,
                        });
                    } else {
                        duplicates.insert(name);
                    }
                }

                self.analyze_inline(function.body());
            }

            Expression::Let { meta: _, init } => self.analyze_inline(init),

            Expression::If {
                condition,
                then,
                or_else,
            } => {
                self.check_condition(expr, condition);
                self.analyze_inline(condition);
                self.analyze_inline(then);
                self.analyze_inline(or_else);
            }

            Expression::PropertyAccess { lhs, property: _ } => {
                self.analyze_inline(lhs);
            }

            Expression::ArrayIndex { lhs, index } => {
                self.analyze_inline(lhs);
                self.analyze_inline(index);

                if let Expression::Number(n) = index.0 {
                    if n < 0. {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::NegativeArrayIndex,
                            severity: Severity::Warning,
                            span: &index.1,
                        });
                    }

                    if n.trunc() != n {
                        self.diagnose(Diagnostic {
                            kind: DiagnosticKind::FractionalArrayIndex,
                            severity: Severity::Warning,
                            span: &index.1,
                        });
                    }
                }
            }

            Expression::BinaryOp {
                lhs,
                operator: _,
                rhs,
            } => {
                self.analyze_inline(lhs);
                self.analyze_inline(rhs);
            }

            Expression::UnaryOp { operator: _, rhs } => {
                self.analyze_inline(rhs);
            }

            Expression::Call {
                function,
                arguments,
            } => {
                self.analyze_inline(function);
                self.analyze_each_inline(arguments.iter());
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
            span: &expr.1,
        });
    }
}

pub fn analyze<'a>(program: &'a Program) -> Diagnoses<'a> {
    Analyzer::new(program).analyze_prog()
}
