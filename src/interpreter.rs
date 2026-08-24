use crate::builtins::{bun_print, bun_println, bun_read};
use crate::environment::Environment;
use crate::error::BunError;
use crate::lexer::TokenKind;
use crate::location::Span;
use crate::parser::{Expr, ExprKind, Item, ItemKind, Stmt, StmtKind};
use crate::value::Value;
use std::cmp::Ordering;
use std::collections::HashMap;

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

        global_env.define("io::println", Value::Native(bun_println));
        global_env.define("io::print", Value::Native(bun_print));
        global_env.define("io::read", Value::Native(bun_read));

        for item in self.items.values() {
            if let ItemKind::Use { path, alias } = &item.value {
                let target = path.join("::");
                let prefix_module = format!("{}::", target);
                let bindings = global_env.get_all();
                let mut found = false;

                for (key, val) in bindings {
                    if key.starts_with(&prefix_module) {
                        let suffix = &key[prefix_module.len()..];
                        let new_key = if let Some(a) = alias {
                            format!("{}::{}", a, suffix)
                        } else {
                            suffix.to_string()
                        };
                        global_env.define(new_key, val);
                        found = true;
                    } else if key == target {
                        let new_key = if let Some(a) = alias {
                            a.clone()
                        } else {
                            path.last().unwrap().clone()
                        };
                        global_env.define(new_key, val);
                        found = true;
                    }
                }

                if !found {
                    return Err(BunError {
                        message: format!("Module or function '{}' not found", target),
                        span: item.span,
                        source: self.code,
                    });
                }
            }
        }

        dbg!(&global_env);

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
            ExprKind::Literal(TokenKind::Float(n)) => Ok(Value::Float(*n)),
            ExprKind::Literal(TokenKind::Int(n)) => Ok(Value::Int(*n)),
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
                        if let (Value::Int(start), Value::Int(end)) = (lhs, rhs) {
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
                self.in_block_scope(env, |block_env| {
                    let mut last_val = Value::Unit;
                    for stmt in stmts {
                        last_val = self.eval_stmt(stmt, block_env)?;
                    }
                    Ok(last_val)
                })
            }
            ExprKind::Call { path, args } => {
                let func_name = path.join("::");
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, env)?);
                }
                if let Some(val) = env.get(&func_name) {
                    if let Value::Native(native_fn) = val {
                        return native_fn(evaluated_args).map_err(|msg| {
                            BunError {
                                message: msg,
                                span: expr.span,
                                source: self.code,
                            }
                            .into()
                        });
                    }
                }
                if let Some(func_item) = self.items.get(path.last().unwrap()) {
                    match &func_item.value {
                        ItemKind::FnDeclare { inputs, block, .. } => {
                            if evaluated_args.len() != inputs.len() {
                                return Err(BunError {
                                    message: format!(
                                        "Function '{}' expects {} arguments, but got {}",
                                        func_name,
                                        inputs.len(),
                                        evaluated_args.len()
                                    ),
                                    span: expr.span,
                                    source: self.code,
                                }
                                .into());
                            }
                            let mut func_env = Environment::new(None);
                            func_env.vars = env.get_top();
                            for (input_name, arg_val) in inputs.iter().zip(evaluated_args) {
                                func_env.define(input_name, arg_val);
                            }
                            return self.eval_block(block, &mut func_env).map_err(|e| e.into());
                        }
                        _ => {}
                    };
                }

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
            ExprKind::For {
                item,
                iterable,
                body,
            } => {
                let iter_val = self.eval_expr(iterable, env)?;
                let mut last_val = Value::Unit;

                if let Value::Range(start, end) = iter_val {
                    let mut current = start;
                    let target = end;
                    while current < target {
                        let res = self.in_block_scope(env, |loop_env| {
                            loop_env.define(item.clone(), Value::Int(current));
                            self.eval_expr(body, loop_env)
                        });
                        match res {
                            Ok(val) => last_val = val,
                            Err(ControlFlow::Break) => break,
                            Err(ControlFlow::Continue) => {}
                            Err(err) => return Err(err),
                        }
                        current += 1;
                    }
                    Ok(last_val)
                } else {
                    Err(BunError {
                        message: format!(
                            "Target of type '{}' is not iterable",
                            iter_val.get_type_name()
                        ),
                        span: expr.span,
                        source: self.code,
                    }
                    .into())
                }
            }
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver_val = self.eval_expr(receiver, env)?;
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, env)?);
                }
                receiver_val
                    .call_method(method, evaluated_args)
                    .map_err(|msg| {
                        BunError {
                            message: msg,
                            span: expr.span,
                            source: self.code,
                        }
                        .into()
                    })
            }
            ExprKind::Break => Err(ControlFlow::Break),
            ExprKind::Continue => Err(ControlFlow::Continue),
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
            StmtKind::Use { path, alias } => {
                let target = path.join("::");
                let prefix_module = format!("{}::", target);
                let bindings = env.get_all();
                let mut found = false;

                for (key, val) in bindings {
                    if key.starts_with(&prefix_module) {
                        let suffix = &key[prefix_module.len()..];
                        let new_key = if let Some(a) = alias {
                            format!("{}::{}", a, suffix)
                        } else {
                            suffix.to_string()
                        };
                        env.define(new_key, val);
                        found = true;
                    } else if key == target {
                        let new_key = if let Some(a) = alias {
                            a.clone()
                        } else {
                            path.last().unwrap().clone()
                        };
                        env.define(new_key, val);
                        found = true;
                    }
                }

                if !found {
                    return Err(ControlFlow::Error(BunError {
                        message: format!("Module or function '{}' not found", target),
                        span: stmt.span,
                        source: self.code,
                    }));
                }

                Ok(Value::Unit)
            }
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

    fn in_block_scope<F>(
        &self,
        env: &mut Environment,
        mut f: F,
    ) -> Result<Value, ControlFlow<'src>>
    where
        F: FnMut(&mut Environment) -> Result<Value, ControlFlow<'src>>,
    {
        let parent_env = std::mem::replace(env, Environment::new(None));
        let mut block_env = Environment::new(Some(parent_env));

        let result = f(&mut block_env);

        *env = block_env.parent.map(|p| *p).unwrap_or(Environment::new(None));
        result
    }
}
