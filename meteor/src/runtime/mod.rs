// pub mod array;
pub mod collections;
pub mod error;
pub mod io;
pub mod memory;
pub mod value;

use std::{borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc};

use error::RuntimeError;
use io::{Sink, Socket};
use memory::{Memory, Variable};
use value::{Function, Type, Value};

use crate::parser::{
    ast::{self, Expression, Program},
    operator::Operator,
    span::{Span, Spanned},
};

pub type Result<T> = std::result::Result<T, error::RuntimeError>;

#[derive(Debug)]
pub struct Chip {
    inputs: Vec<Sink>,
    outputs: Vec<Socket>,
    program: Rc<Program>,
    memory: Memory,
}

impl Chip {
    pub fn new(program: Program) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
            memory: Memory::new(),
            program: Rc::new(program),
        }
    }

    pub fn run(&mut self) -> Result<Value> {
        let programs = self.program.clone();

        for p in programs.expressions().iter() {
            self.eval(p)?;
        }

        let main = "main".into();

        let func = self
            .memory
            .retrieve(&main, &(Expression::Ident(main.clone()), Span::empty()))?;

        match func.value() {
            Value::Function(func) => self.run_func(func, &[]),
            _ => Err(RuntimeError::InvalidMainFunc),
        }
    }

    fn resolve_type(&self, ty: &Spanned<ast::Type>) -> Result<Type> {
        match &ty.0 {
            ast::Type::Named(identifier) => Ok({
                match identifier.name() {
                    "int" | "float" | "number" => Type::Number,
                    "bool" => Type::Bool,
                    "str" => Type::String,
                    "nil" => Type::Nil,
                    "any" => Type::Any,
                    "dict" => Type::Dictionary {
                        key: Box::new(Type::Any),
                        value: Box::new(Type::Any),
                    },
                    "array" => Type::Array(Box::new(Type::Any)),

                    // TODO: support custom types
                    _ => {
                        return Err(RuntimeError::UnknownType {
                            data_type: ty.clone(),
                        })
                    }
                }
            }),
            ast::Type::Generic(..) => todo!(),
        }
    }

    fn eval(&mut self, expr: &Spanned<Expression>) -> Result<Value> {
        Ok(match &expr.0 {
            Expression::Nil => Value::Nil,
            Expression::Ident(identifier) => {
                self.memory.retrieve(&identifier, expr)?.value().clone()
            }
            Expression::String(str) => Value::String(str.clone()),
            Expression::Bool(b) => Value::Bool(*b),
            Expression::Number(n) => Value::Number(*n),
            Expression::Array(vec) => Value::Array(Rc::new(
                vec.iter()
                    .try_fold(Vec::with_capacity(vec.len()), |mut v, x| {
                        v.push(self.eval(x)?);
                        Ok(v)
                    })?
                    .into(),
            )),

            Expression::Func(function) => Value::Function(Rc::new(Function::new(
                Rc::new(ast::Function::clone(function)),
                self.memory.clone(),
            ))),

            Expression::Let { meta, init } => {
                let (meta, ..) = meta;

                let data_type = if let Some(ty) = meta.data_type() {
                    self.resolve_type(ty)?
                } else {
                    Default::default()
                };

                let value = self.eval(init)?;

                self.memory.define(
                    meta.name().clone(),
                    Variable {
                        data_type,
                        mutability: meta.mutablity(),
                        value,
                    },
                    expr,
                )?;

                Value::Nil
            }

            Expression::Block(vec) => {
                let mut v = Value::Nil;

                self.memory.push_env();
                for expr in vec.iter() {
                    v = self.eval(expr)?;
                }
                self.memory.pop_env();

                v
            }

            Expression::If {
                condition,
                then,
                or_else,
            } => {
                let condition = self.eval(condition)?;

                self.eval(if condition.truthy() { then } else { or_else })?
            }

            Expression::While { condition, then } => {
                while self.eval(condition)?.truthy() {
                    self.eval(then)?;
                }

                Value::Nil
            }

            Expression::PropertyAccess { lhs, property } => match self.eval(lhs)? {
                Value::Dictionary(values) => {
                    values.get(property).map(Value::clone).unwrap_or(Value::Nil)
                }
                Value::Array(..)
                | Value::String(..)
                | Value::Bool(..)
                | Value::Number(..)
                | Value::Function(..)
                | Value::Nil => {
                    return Err(RuntimeError::InvalidPropertyAccess {
                        obj: *lhs.clone(),
                        property: property.clone(),
                    })
                }
            },

            Expression::ArrayIndex { lhs, index } => match (self.eval(lhs)?, self.eval(index)?) {
                (Value::Array(arr), Value::Number(i)) => RefCell::borrow(&arr)
                    .get(i.floor() as usize)
                    .ok_or_else(|| RuntimeError::ArrayOutOfBounds {
                        array: *lhs.clone(),
                        index: *index.clone(),
                    })?
                    .clone(),

                (obj, _) => {
                    return Err(RuntimeError::CannotIndexIntoType {
                        array: *lhs.clone(),
                        data_type: obj.get_type(),
                    })
                }
            },

            Expression::BinaryOp { lhs, operator, rhs } => {
                let unsupported = || Err(RuntimeError::UnsupportedOperation(expr.clone()));

                if operator == &Operator::Assign {
                    let value = self.eval(rhs)?;

                    match &lhs.0 {
                        Expression::Ident(ref ident) => {
                            self.memory.store(ident, value, expr)?;
                            return Ok(Value::Nil);
                        }
                        Expression::ArrayIndex { lhs, index } => {
                            match (self.eval(lhs)?, self.eval(index)?) {
                                (Value::Array(arr), Value::Number(n)) => {
                                    arr.borrow_mut()[n as usize] = value;
                                    return Ok(Value::Nil);
                                }
                                _ => {}
                            }
                        }

                        _ => {}
                    }

                    return unsupported();
                }

                match (self.eval(lhs)?, self.eval(rhs)?) {
                    (Value::Number(a), Value::Number(b)) => Value::Number(match operator {
                        Operator::Sub => a - b,
                        Operator::Add => a + b,
                        Operator::Mul => a * b,
                        Operator::Div => a / b,
                        Operator::Mod => a % b,
                        Operator::Equals => return Ok(Value::Bool(a == b)),
                        Operator::NotEqual => return Ok(Value::Bool(a != b)),
                        Operator::Greater => return Ok(Value::Bool(a > b)),
                        Operator::GreaterOrEqual => return Ok(Value::Bool(a >= b)),
                        Operator::Less => return Ok(Value::Bool(a < b)),
                        Operator::LessOrEqual => return Ok(Value::Bool(a <= b)),
                        _ => return unsupported(),
                    }),
                    (Value::Bool(a), Value::Bool(b)) => Value::Bool(match operator {
                        Operator::Or => a || b,
                        Operator::And => a && b,
                        Operator::Nor => !a && !b,
                        Operator::Xor => a ^ b,
                        Operator::Equals => a == b,
                        Operator::NotEqual => a != b,

                        _ => todo!(),
                    }),
                    _ => Value::Nil,
                }
            }

            Expression::UnaryOp { operator, rhs } => {
                debug_assert!(
                    operator.is_unary(),
                    "Operator for unary expression must be unary"
                );

                let value = self.eval(rhs)?;

                match (operator, value) {
                    (Operator::Sub, Value::Number(n)) => Value::Number(-n),
                    (Operator::Not, Value::Bool(b)) => Value::Bool(!b),

                    (_, value) => {
                        return Err(RuntimeError::UnsupportedUnaryOperation(
                            *operator,
                            *(rhs.clone()),
                            value.clone(),
                        ))
                    }
                }
            }

            Expression::Call {
                function,
                arguments,
            } => {
                if matches!(function.0, Expression::Ident(ref ident) if **ident == *"print" ) {
                    for arg in arguments {
                        println!("{}", self.eval(arg)?);
                    }
                    return Ok(Value::Nil);
                }

                match self.eval(function)? {
                    Value::Function(ref func) => self.run_func(func, arguments)?,

                    _ => todo!(),
                }
            }

            Expression::Dictionary(vec) => {
                let mut map = HashMap::new();

                for (key, val) in vec {
                    map.insert(key.clone(), self.eval(val)?);
                }

                Value::Dictionary(Rc::new(map))
            }

            Expression::Error => Value::Nil,
        })
    }

    fn run_func(
        &mut self,
        func: &Rc<Function>,
        arguments: &[Spanned<Expression>],
    ) -> Result<Value> {
        // Push memory scope
        let old = std::mem::replace(&mut self.memory, func.scope().clone());

        let func = func.inner();

        self.memory.push_env();

        for (param, arg) in func.arguments().iter().zip(arguments) {
            let data_type = if let Some(ty) = param.0.data_type() {
                self.resolve_type(ty)?
            } else {
                Default::default()
            };

            let value = self.eval(arg)?;

            self.memory.define(
                param.0.name().clone(),
                Variable {
                    data_type,
                    mutability: param.0.mutablity(),
                    value,
                },
                arg,
            )?;
        }

        let returns = self.eval(func.body())?;

        let _ = self.memory.pop_env();

        // Pop memory scope
        self.memory = old;

        Ok(returns)
    }
}
