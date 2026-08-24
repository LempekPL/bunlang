use crate::environment::Environment;
use crate::error::BunError;
use crate::lexer::TokenKind;
use crate::location::Span;
use crate::parser::{Expr, ExprKind, Item, ItemKind, Stmt, StmtKind};
pub(crate) use crate::value::Value;
use std::collections::HashMap;

pub struct Interpreter<'src> {
    code: &'src str,
}

impl<'src> Interpreter<'src> {
    pub fn new(code: &'src str) -> Self {
        Self { code }
    }

    pub fn run(items: HashMap<String, Item>, code: &'src str) -> Result<Value, BunError<'src>> {
        let interpreter = Interpreter::new(code);
        let mut global_env = Environment::new(None);

        let block = if let Some(Item {
            value: ItemKind::FnDeclare { block, .. },
            ..
        }) = items.get("main")
        {
            block.clone()
        } else {
            return Err(BunError {
                message: "Function 'main' not found".into(),
                span: Span::default(),
                source: code,
            });
        };

        interpreter.eval_expr(&block, &mut global_env)
    }

    pub fn eval_expr(&self, expr: &Expr, env: &mut Environment) -> Result<Value, BunError<'src>> {
        match &expr.value {
            ExprKind::Literal(TokenKind::Number(n)) => Ok(Value::F64(*n)),
            ExprKind::Literal(TokenKind::String(s)) => Ok(Value::String(s.clone())),
            ExprKind::Literal(TokenKind::Bool(b)) => Err(BunError {
                message: format!("Boolean literals not supported in Value yet (found {})", b),
                span: expr.span,
                source: self.code,
            }),
            ExprKind::Ident(name) => env.get(name).ok_or_else(|| BunError {
                message: format!("Undefined variable '{}'", name),
                span: expr.span,
                source: self.code,
            }),
            ExprKind::Unary { op, expr } => {
                let val = self.eval_expr(expr, env)?;
                match op {
                    TokenKind::Minus => (-val).map_err(|e| BunError {
                        message: e.message.to_string(),
                        span: expr.span,
                        source: self.code,
                    }),
                    _ => Err(BunError {
                        message: format!("Unsupported unary operator '{:?}'", op),
                        span: expr.span,
                        source: self.code,
                    }),
                }
            }
            ExprKind::Binary { op, left, right } => {
                let lhs = self.eval_expr(left, env)?;
                let rhs = self.eval_expr(right, env)?;

                let res = match op {
                    TokenKind::Plus => lhs + rhs,
                    TokenKind::Minus => lhs - rhs,
                    TokenKind::Star => lhs * rhs,
                    TokenKind::Slash => lhs / rhs,
                    TokenKind::Percent => lhs % rhs,
                    _ => {
                        return Err(BunError {
                            message: format!("Unsupported binary operator '{:?}'", op),
                            span: expr.span,
                            source: self.code,
                        });
                    }
                };

                res.map_err(|e| BunError {
                    message: e.message.to_string(),
                    span: expr.span,
                    source: self.code,
                })
            }
            ExprKind::Assign { target, value } => {
                let val = self.eval_expr(value, env)?;
                if let ExprKind::Ident(name) = &target.value {
                    env.assign(name, val.clone()).map_err(|msg| BunError {
                        message: msg,
                        span: target.span,
                        source: self.code,
                    })?;
                    Ok(val)
                } else {
                    Err(BunError {
                        message: "Invalid assignment target".to_string(),
                        span: target.span,
                        source: self.code,
                    })
                }
            }
            ExprKind::Block { stmts } => {
                let parent_env = std::mem::replace(env, Environment::new(None));
                let mut block_env = Environment::new(Some(parent_env));
                let mut last_val = Value::Unit;
                for stmt in stmts {
                    last_val = self.eval_stmt(stmt, &mut block_env)?;
                }
                *env = block_env.parent.map(|p| *p).unwrap_or(Environment::new(None));

                Ok(last_val)
            }
            ExprKind::Call { path, args } => {
                let func_name = path.join("::");
                if func_name == "io::println" {
                    for arg in args {
                        let val = self.eval_expr(arg, env)?;
                        print!("{}", val);
                    }
                    println!();
                    return Ok(Value::Unit);
                }
                Err(BunError {
                    message: format!("Unknown function '{}'", func_name),
                    span: expr.span,
                    source: self.code,
                })
            }
            _ => Err(BunError {
                message: "Expression not implemented in interpreter yet".to_string(),
                span: expr.span,
                source: self.code,
            }),
        }
    }

    pub fn eval_stmt(&self, stmt: &Stmt, env: &mut Environment) -> Result<Value, BunError<'src>> {
        match &stmt.value {
            StmtKind::Declare { name, expr } => {
                let val = self.eval_expr(expr, env)?;
                env.define(name, val);
                Ok(Value::Unit)
            }
            StmtKind::Expr(expr) => self.eval_expr(expr, env),
        }
    }
}
