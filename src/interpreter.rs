use crate::environment::Environment;
use crate::error::BunError;
use crate::lexer::TokenKind;
use crate::location::Span;
use crate::parser::{Expr, ExprKind, Item, ItemKind, Stmt, StmtKind};
pub(crate) use crate::value::Value;
use std::collections::HashMap;

pub struct Interpreter<'src> {
    env: Environment,
    code: &'src str,
}

impl<'src> Interpreter<'src> {
    pub fn run(items: HashMap<String, Item>, code: &'src str) -> Result<Value, BunError<'src>> {
        let mut interpreter = Interpreter {
            env: Environment::new(None),
            code,
        };
        if let Some(Item {
            value: ItemKind::FnDeclare { block, .. },
            ..
        }) = items.get("main")
        {
            interpreter.interpret_expr(block)
        } else {
            Err(BunError {
                message: "Function 'main' not found".into(),
                span: Span::default(),
                source: &code,
            })
        }
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, BunError<'src>> {
        match &expr.value {
            ExprKind::Literal(v) => match v {
                TokenKind::Number(v) => Ok(Value::Number(*v)),
                TokenKind::String(v) => Ok(Value::String(v.clone())),
                _ => Err(BunError {
                    message: "Unexpected literal value".into(),
                    span: expr.span.clone(),
                    source: &self.code,
                }),
            },
            ExprKind::Ident(v) => self.env.get(v).ok_or(BunError {
                message: format!("Undefined variable '{}'", v),
                span: expr.span.clone(),
                source: &self.code,
            }),
            ExprKind::Binary { op, left, right } => {
                let left = self.interpret_expr(left)?;
                let right = self.interpret_expr(right)?;
                match op {
                    TokenKind::Plus => left + right,
                    TokenKind::Minus => left - right,
                    TokenKind::Star => left * right,
                    TokenKind::Slash => left / right,
                    TokenKind::Percent => left % right,
                    _ => unreachable!(),
                }
            }
            ExprKind::Unary { op, expr } => {
                let expr = self.interpret_expr(expr)?;
                match op {
                    TokenKind::Minus => -expr,
                    _ => unreachable!(),
                }
            }
            ExprKind::Block { stmts } => {
                for stmt in stmts {
                    self.interpret_stmt(stmt)?;
                }
                Ok(Value::Unit)
            }
            // ExprKind::Call { .. } => {}
            _ => {
                todo!()
            }
        }
    }

    fn interpret_stmt(&mut self, stmt: &Stmt) -> Result<Value, BunError<'src>> {
        match &stmt.value {
            StmtKind::Declare { name, expr } => {
                let expr = self.interpret_expr(&expr)?;
                self.env.set(name, expr);
                Ok(Value::Unit)
            }
            StmtKind::Assign { name, expr } => {
                let expr = self.interpret_expr(&expr)?;
                self.env.set(name, expr);
                Ok(Value::Unit)
            }
            StmtKind::Call { name, args } => {
                match name.as_str() {
                    "println" => {
                        let expr = self.interpret_expr(&args[0])?;
                        println!("{}", expr);
                    }
                    _ => {
                        todo!()
                    }
                }
                Ok(Value::Unit)
            }
        }
    }

    fn nest(&mut self) {
        let parent = std::mem::replace(&mut self.env, Environment::new(None));
        self.env = Environment::new(Some(parent));
    }
}
