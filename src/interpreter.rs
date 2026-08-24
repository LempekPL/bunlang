use crate::environment::Environment;
use crate::error::BunError;
use crate::lexer::TokenKind;
use crate::location::Span;
use crate::parser::{Expr, ExprKind, Item, ItemKind, Stmt, StmtKind};
pub(crate) use crate::value::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;

#[derive(Debug)]
pub enum ControlFlow<'src> {
    Error(BunError<'src>),
    Break,
    Continue,
}

impl<'src> From<BunError<'src>> for ControlFlow<'src> {
    fn from(err: BunError<'src>) -> Self {
        ControlFlow::Error(err)
    }
}

pub struct Interpreter<'src> {
    items: HashMap<String, Item>,
    code: &'src str,
}

impl<'src> Interpreter<'src> {
    pub fn new(items: HashMap<String, Item>, code: &'src str) -> Self {
        Self { items, code }
    }

    pub fn run(&mut self) -> Result<(), BunError<'src>> {
        let mut global_env = Environment::new(None);

        let Some(Item {
            value: ItemKind::FnDeclare { block, .. },
            ..
        }) = self.items.get("main")
        else {
            return Err(BunError {
                message: "Function 'main' not found".into(),
                span: Span::default(),
                source: self.code,
            });
        };
        self.eval_block(block, &mut global_env)?;
        Ok(())
    }

    pub fn eval_expr(
        &self,
        expr: &Expr,
        env: &mut Environment,
    ) -> Result<Value, ControlFlow<'src>> {
        match &expr.value {
            ExprKind::Literal(TokenKind::Bool(b)) => Ok(Value::Bool(*b)),
            ExprKind::Literal(TokenKind::Number(n)) => Ok(Value::F64(*n)),
            ExprKind::Literal(TokenKind::String(s)) => Ok(Value::String(s.clone())),
            ExprKind::Ident(name) => env.get(name).ok_or_else(|| {
                BunError {
                    message: format!("Undefined variable '{}'", name),
                    span: expr.span,
                    source: self.code,
                }
                .into()
            }),
            ExprKind::Unary {
                op,
                expr: inner_expr,
            } => {
                let val = self.eval_expr(inner_expr, env)?;
                let res = match op {
                    TokenKind::Minus => -val,
                    TokenKind::Not => !val,
                    _ => Err(format!("Unsupported unary operator '{:?}'", op)),
                };
                res.map_err(|msg| {
                    BunError {
                        message: msg,
                        span: expr.span,
                        source: self.code,
                    }
                    .into()
                })
            }
            ExprKind::Binary { op, left, right } => {
                let lhs = self.eval_expr(left, env)?;
                if *op == TokenKind::And {
                    let lhs_bool = lhs.is_true().map_err(|msg| BunError {
                        message: msg,
                        span: left.span,
                        source: self.code,
                    })?;
                    if !lhs_bool {
                        return Ok(Value::Bool(false));
                    }

                    let rhs = self.eval_expr(right, env)?;
                    let rhs_bool = rhs.is_true().map_err(|msg| BunError {
                        message: msg,
                        span: right.span,
                        source: self.code,
                    })?;
                    return Ok(Value::Bool(rhs_bool));
                } else if *op == TokenKind::Or {
                    let lhs_bool = lhs.is_true().map_err(|msg| BunError {
                        message: msg,
                        span: left.span,
                        source: self.code,
                    })?;
                    if lhs_bool {
                        return Ok(Value::Bool(true));
                    }

                    let rhs = self.eval_expr(right, env)?;
                    let rhs_bool = rhs.is_true().map_err(|msg| BunError {
                        message: msg,
                        span: right.span,
                        source: self.code,
                    })?;
                    return Ok(Value::Bool(rhs_bool));
                }

                let rhs = self.eval_expr(right, env)?;

                let res = match op {
                    TokenKind::Plus => lhs + rhs,
                    TokenKind::Minus => lhs - rhs,
                    TokenKind::Star => lhs * rhs,
                    TokenKind::Slash => lhs / rhs,
                    TokenKind::Percent => lhs % rhs,
                    TokenKind::EqualsEquals => Ok(Value::Bool(lhs == rhs)),
                    TokenKind::NotEquals => Ok(Value::Bool(lhs != rhs)),
                    TokenKind::Less => lhs
                        .partial_cmp(&rhs)
                        .map(|ord| Value::Bool(ord == Ordering::Less))
                        .ok_or_else(|| format!("Cannot compare {:?} and {:?}", lhs, rhs)),
                    TokenKind::LessEquals => lhs
                        .partial_cmp(&rhs)
                        .map(|ord| Value::Bool(ord == Ordering::Less || ord == Ordering::Equal))
                        .ok_or_else(|| format!("Cannot compare {:?} and {:?}", lhs, rhs)),
                    TokenKind::Greater => lhs
                        .partial_cmp(&rhs)
                        .map(|ord| Value::Bool(ord == Ordering::Greater))
                        .ok_or_else(|| format!("Cannot compare {:?} and {:?}", lhs, rhs)),
                    TokenKind::GreaterEquals => lhs
                        .partial_cmp(&rhs)
                        .map(|ord| Value::Bool(ord == Ordering::Greater || ord == Ordering::Equal))
                        .ok_or_else(|| format!("Cannot compare {:?} and {:?}", lhs, rhs)),
                    TokenKind::DotDot => {
                        if let (Value::F64(start), Value::F64(end)) = (lhs, rhs) {
                            Ok(Value::Range(start, end))
                        } else {
                            Err("Range bounds must be numbers".to_string())
                        }
                    }
                    _ => {
                        return Err(BunError {
                            message: format!("Unsupported binary operator '{:?}'", op),
                            span: expr.span,
                            source: self.code,
                        }
                        .into());
                    }
                };

                res.map_err(|msg| {
                    BunError {
                        message: msg,
                        span: expr.span,
                        source: self.code,
                    }
                    .into()
                })
            }
            ExprKind::Assign { op, target, value } => {
                let right_val = self.eval_expr(value, env)?;

                if let ExprKind::Ident(name) = &target.value {
                    let final_val = if *op == TokenKind::Equals {
                        right_val
                    } else {
                        let current_val = env.get(name).ok_or_else(|| BunError {
                            message: format!("Undefined variable '{}'", name),
                            span: target.span,
                            source: self.code,
                        })?;

                        let res = match op {
                            TokenKind::PlusEquals => current_val + right_val,
                            TokenKind::MinusEquals => current_val - right_val,
                            TokenKind::StarEquals => current_val * right_val,
                            TokenKind::SlashEquals => current_val / right_val,
                            TokenKind::PercentEquals => current_val % right_val,
                            _ => Err(format!("Unsupported assignment operator '{:?}'", op)),
                        };

                        res.map_err(|msg| BunError {
                            message: msg,
                            span: expr.span,
                            source: self.code,
                        })?
                    };
                    env.assign(name, final_val.clone())
                        .map_err(|msg| BunError {
                            message: msg,
                            span: target.span,
                            source: self.code,
                        })?;
                    Ok(final_val)
                } else {
                    Err(BunError {
                        message: "Invalid assignment target".to_string(),
                        span: target.span,
                        source: self.code,
                    }
                    .into())
                }
            }
            ExprKind::Block { stmts } => {
                let parent_env = std::mem::replace(env, Environment::new(None));
                let mut block_env = Environment::new(Some(parent_env));
                let mut last_val = Value::Unit;
                let mut flow_err = None;
                for stmt in stmts {
                    match self.eval_stmt(stmt, &mut block_env) {
                        Ok(val) => last_val = val,
                        Err(e) => {
                            flow_err = Some(e);
                            break;
                        }
                    }
                }
                *env = block_env
                    .parent
                    .map(|p| *p)
                    .unwrap_or(Environment::new(None));

                if let Some(err) = flow_err {
                    return Err(err);
                }

                Ok(last_val)
            }
            ExprKind::Call { path, args } => {
                if path[0] == "io" {
                    match path[1].as_str() {
                        "println" => {
                            for arg in args {
                                let val = self.eval_expr(arg, env)?;
                                print!("{}", val);
                            }
                            println!();
                            return Ok(Value::Unit);
                        }
                        "print" => {
                            for arg in args {
                                let val = self.eval_expr(arg, env)?;
                                print!("{}", val);
                            }
                            return Ok(Value::Unit);
                        }
                        "read" => {
                            if args.len() > 1 {
                                return Err(BunError {
                                    message: "Read doesn't take more than 0 arguments".to_string(),
                                    span: expr.span,
                                    source: self.code,
                                }
                                .into());
                            }
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).expect("Failed to read input");
                            return Ok(Value::String(input.trim_end().to_string()));
                        }
                        _ => {}
                    }
                }
                if let Some(func_name) = path.last()
                    && let Some(func_item) = self.items.get(func_name)
                {
                    match &func_item.value {
                        ItemKind::FnDeclare {
                            inputs,
                            block,
                            ..
                        } => {
                            if args.len() != inputs.len() {
                                return Err(BunError {
                                    message: format!(
                                        "Function '{}' expects {} arguments, but got {}",
                                        func_name,
                                        inputs.len(),
                                        args.len()
                                    ),
                                    span: expr.span,
                                    source: self.code,
                                }
                                    .into());
                            }
                            let mut func_env = Environment::new(None);
                            for (input_name, args) in inputs.iter().zip(args) {
                                func_env.define(input_name, self.eval_expr(args, env)?);
                            }
                            return self.eval_block(block, &mut func_env).map_err(|e| e.into());
                        }
                    };
                }
                let func_name = path.join("::");
                Err(BunError {
                    message: format!("Unknown function '{}'", func_name),
                    span: expr.span,
                    source: self.code,
                }
                .into())
            }
            ExprKind::If {
                condition,
                truthy,
                falsy,
            } => {
                let cond_val =
                    self.eval_expr(condition, env)?
                        .is_true()
                        .map_err(|err| BunError {
                            message: err,
                            span: expr.span,
                            source: self.code,
                        })?;
                if cond_val {
                    self.eval_expr(truthy, env)
                } else if let Some(falsy_expr) = falsy {
                    self.eval_expr(falsy_expr, env)
                } else {
                    Ok(Value::Unit)
                }
            }

            ExprKind::While { condition, body } => {
                let mut last_val = Value::Unit;

                loop {
                    let cond_val = self.eval_expr(condition, env)?;
                    let is_true = cond_val.is_true().map_err(|msg| BunError {
                        message: msg,
                        span: condition.span,
                        source: self.code,
                    })?;

                    if !is_true {
                        break;
                    }
                    match self.eval_expr(body, env) {
                        Ok(val) => last_val = val,
                        Err(ControlFlow::Break) => break,
                        Err(ControlFlow::Continue) => continue,
                        Err(err) => return Err(err),
                    }
                }

                Ok(last_val)
            }

            // ExprKind::For {
            //     item,
            //     iterable,
            //     body,
            // } => {
            //     let iter_val = self.eval_expr(iterable, env)?;
            //     let mut last_val = Value::Unit;
            //
            //     // Currently only supporting our internal Range type
            //     if let Value::Range(start, end) = iter_val {
            //         let mut current = start as i64;
            //         let target = end as i64;
            //
            //         while current < target {
            //             // Create a new scope for the loop body
            //             let parent_env = std::mem::replace(env, Environment::new(None));
            //             let mut loop_env = Environment::new(Some(parent_env));
            //
            //             // Define the iterator variable (e.g., 'i' in 'for i in 0..10')
            //             loop_env.define(item.clone(), Value::F64(current as f64));
            //
            //             last_val = self.eval_stmt(body, &mut loop_env)?;
            //
            //             // Restore parent environment
            //             *env = loop_env
            //                 .parent
            //                 .map(|p| *p)
            //                 .unwrap_or(Environment::new(None));
            //             current += 1;
            //         }
            //         Ok(last_val)
            //     } else {
            //         Err(BunError {
            //             message: "Target is not iterable".to_string(),
            //             span: expr.span,
            //             source: self.code,
            //         })
            //     }
            // }
            ExprKind::Break => return Err(ControlFlow::Break),
            ExprKind::Continue => return Err(ControlFlow::Continue),
            _ => Err(BunError {
                message: "Expression not implemented in interpreter yet".to_string(),
                span: expr.span,
                source: self.code,
            }
            .into()),
        }
    }

    pub fn eval_stmt(
        &self,
        stmt: &Stmt,
        env: &mut Environment,
    ) -> Result<Value, ControlFlow<'src>> {
        match &stmt.value {
            StmtKind::Declare { name, expr } => {
                let val = self.eval_expr(expr, env)?;
                env.define(name, val);
                Ok(Value::Unit)
            }
            StmtKind::Expr(expr) => self.eval_expr(expr, env),
        }
    }

    pub fn eval_block(&self, block: &Expr, env: &mut Environment) -> Result<Value, BunError<'src>> {
        match self.eval_expr(block, env) {
            Ok(value) => Ok(value),
            Err(ControlFlow::Error(err)) => Err(err),
            Err(ControlFlow::Break) => Err(BunError {
                message: "Cannot use 'break' outside of a loop".into(),
                span: block.span,
                source: self.code,
            }),
            Err(ControlFlow::Continue) => Err(BunError {
                message: "Cannot use 'continue' outside of a loop".into(),
                span: block.span,
                source: self.code,
            }),
        }
    }
}
