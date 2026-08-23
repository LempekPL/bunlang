use std::collections::HashMap;
use crate::lexer::{Keyword, Token, TokenKind};
use crate::location::{Locatable, Pos, Span};
use std::fmt;
use std::marker::PhantomData;
use crate::error::BunError;

#[derive(Debug)]
pub enum ExprKind {
    Literal(TokenKind),
    Ident(String),
    Binary {
        op: TokenKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: TokenKind,
        expr: Box<Expr>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    QualifiedIdent(Vec<String>),
}

pub type Expr = Locatable<ExprKind>;

#[derive(Debug)]
pub enum StmtKind {
    Declare { name: String, expr: Expr },
    Assign { name: String, expr: Expr },
    Call { name: String, args: Vec<Expr> },
}

pub type Stmt = Locatable<StmtKind>;

#[derive(Debug)]
pub enum ItemKind {
    FnDeclare {
        name: String,
        inputs: Vec<Expr>,
        block: Expr,
    },
}

pub type Item = Locatable<ItemKind>;

pub struct Parser<'src> {
    tokens: Vec<Token>,
    i: usize,
    code: &'src str,
    span: Span,
}

pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl<'src> Parser<'src> {
    pub fn parse(tokens: Vec<Token>, code: &'src str) -> Result<HashMap<String, Item>, BunError<'src>> {
        let mut parser = Self {
            tokens,
            code,
            i: 0,
            span: Span::default(),
        };
        parser.parse_items()
    }

    fn parse_items(&mut self) -> Result<HashMap<String, Item>, BunError<'src>> {
        let mut items = HashMap::new();
        while let Some(tok) = self.current() {
            match &tok.value {
                TokenKind::Keyword(Keyword::Fn) => {
                    let start = tok.span.start;
                    self.advance();
                    let ident = self.take_ident()?;
                    self.consume(TokenKind::LeftParen)?;
                    // todo: implement input
                    self.consume(TokenKind::RightParen)?;
                    let stmts = self.parse_stmts()?;
                    let span = Span::from((start, stmts.span.end));
                    let fun = ItemKind::FnDeclare {
                        name: ident.clone(),
                        inputs: vec![],
                        block: stmts,
                    };
                    items.insert(ident, Item::new_span(fun, span));
                }
                other => {
                    return Err(BunError {
                        message: format!("Unexpected token '{}'", other),
                        span: tok.span,
                        source: self.code,
                    });
                }
            }
        }
        Ok(items)
    }

    fn parse_stmts(&mut self) -> Result<Expr, BunError<'src>> {
        let mut stmts = Vec::new();
        let start = self.span.start;
        self.consume(TokenKind::LeftBrace)?;
        while let Some(tok) = self.current().cloned() {
            let start = tok.span.start;
            match &tok.value {
                TokenKind::Keyword(Keyword::Let) => {
                    self.advance();
                    let name = self.take_ident()?;
                    self.consume(TokenKind::Equals)?;
                    let expr = self.parse_expr()?;
                    self.consume(TokenKind::Semicolon)?;
                    let span = Span::from((start, expr.span.end));
                    let decl = StmtKind::Declare { name, expr };
                    stmts.push(Stmt::new_span(decl, span));
                }
                TokenKind::Ident(_) => {
                    let name = self.take_ident()?;
                    if let Some(next) = self.current().cloned() {
                        match &next.value {
                            TokenKind::Equals => {
                                self.advance();
                                let expr = self.parse_expr()?;
                                self.consume(TokenKind::Semicolon)?;
                                let span = Span::from((start, expr.span.end));
                                let ass = StmtKind::Assign { name, expr };
                                stmts.push(Stmt::new_span(ass, span));
                            }
                            TokenKind::ColonColon => {
                                self.advance();
                                let expr = self.parse_expr()?;
                                self.consume(TokenKind::Semicolon)?;
                                let span = Span::from((start, expr.span.end));
                                let ass = StmtKind::Assign { name, expr };
                                stmts.push(Stmt::new_span(ass, span));
                            }
                            TokenKind::LeftParen => {
                                self.advance();
                                let mut args = Vec::new();
                                while let Some(peek) = self.current() {
                                    match peek.value {
                                        TokenKind::RightParen => {
                                            self.consume(TokenKind::RightParen)?;
                                            break;
                                        }
                                        TokenKind::Comma => {
                                            self.consume(TokenKind::Comma)?;
                                        }
                                        _ => {
                                            let expr = self.parse_expr()?;
                                            args.push(expr);
                                        }
                                    }
                                }
                                let call = StmtKind::Call {
                                    name: name.to_string(),
                                    args,
                                };
                                self.consume(TokenKind::Semicolon)?;
                                let span = Span::from((start, self.span.end));
                                stmts.push(Stmt::new_span(call, span));
                            }
                            other => {
                                return Err(BunError {
                                    message: format!("Unexpected token '{}'", other),
                                    span: next.span,
                                    source: self.code,
                                });
                            }
                        }
                    }
                }
                TokenKind::RightBrace => {
                    break;
                }
                other => {
                    return Err(BunError {
                        message: format!("Unexpected token '{}'", other),
                        span: tok.span,
                        source: self.code,
                    });
                }
            }
        }
        self.consume(TokenKind::RightBrace)?;
        Ok(Expr::new_span(
            ExprKind::Block { stmts },
            Span::from((start, self.span.end)),
        ))
    }

    fn parse_expr(&mut self) -> Result<Expr, BunError<'src>> {
        let expr = self.parse_additive()?;

        Ok(expr)
    }

    fn parse_binary<F>(
        &mut self,
        mut parse_next: F,
        ops: &[TokenKind],
    ) -> Result<Expr, BunError<'src>>
    where
        F: FnMut(&mut Self) -> Result<Expr, BunError<'src>>,
    {
        let mut expr = parse_next(self)?;

        while let Some(tok) = self.current() {
            if ops.contains(&tok.value) {
                let op = tok.value.clone();
                self.advance();

                let right = parse_next(self)?;
                let span = Span::from((expr.span.start, right.span.end));
                expr = Expr::new_span(
                    ExprKind::Binary {
                        op,
                        left: Box::new(expr),
                        right: Box::new(right),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, BunError<'src>> {
        self.parse_binary(
            |s| s.parse_multiplicative(),
            &[TokenKind::Plus, TokenKind::Minus],
        )
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, BunError<'src>> {
        self.parse_binary(
            |s| s.parse_factor(),
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent],
        )
    }

    fn parse_factor(&mut self) -> Result<Expr, BunError<'src>> {
        if let Some(tok) = self.current().cloned() {
            self.advance();
            match &tok.value {
                TokenKind::Number(_) | TokenKind::String(_) => {
                    Ok(Expr::new_span(ExprKind::Literal(tok.value), tok.span))
                }
                TokenKind::Ident(name) => {
                    if let Some(next) = self.current() {
                        match next.value {
                            TokenKind::LeftParen => {
                                let start = tok.span.start;
                                self.advance();
                                let mut args = Vec::new();
                                while let Some(peek) = self.current() {
                                    match peek.value {
                                        TokenKind::RightParen => {
                                            self.consume(TokenKind::RightParen)?;
                                            break;
                                        }
                                        TokenKind::Comma => {
                                            self.consume(TokenKind::Comma)?;
                                        }
                                        _ => {
                                            let expr = self.parse_expr()?;
                                            args.push(expr);
                                        }
                                    }
                                }
                                let call = ExprKind::Call {
                                    name: name.to_string(),
                                    args,
                                };
                                let span = Span::from((start, self.span.end));
                                Ok(Expr::new_span(call, span))
                            }
                            _ => Ok(Expr::new_span(ExprKind::Ident(name.clone()), tok.span))
                        }
                    } else {
                        Err(BunError {
                            message: "Unexpected end of code".into(),
                            span: Span::from(Pos::from((1usize, 1usize))),
                            source: &self.code,
                        })
                    }
                }
                TokenKind::Minus => {
                    let expr = self.parse_factor()?;
                    let end = expr.span.end;
                    Ok(Expr::new_span(
                        ExprKind::Unary {
                            op: TokenKind::Minus,
                            expr: Box::new(expr),
                        },
                        Span::from((tok.span.start, end)),
                    ))
                }
                TokenKind::LeftParen => {
                    let expr = self.parse_expr()?;
                    self.consume(TokenKind::RightParen)?;
                    Ok(expr)
                }
                other => Err(BunError {
                    message: format!("Unexpected token '{}'", other),
                    span: tok.span,
                    source: self.code,
                }),
            }
        } else {
            Err(BunError {
                message: "Unexpected end of code".into(),
                span: Span::from(Pos::from((1usize, 1usize))),
                source: &self.code,
            })
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<(), BunError<'src>> {
        if self.current().is_some_and(|v| v.value == kind) {
            self.advance();
            return Ok(());
        }
        Err(BunError {
            message: format!("Expected '{}'", kind),
            span: self.span,
            source: self.code,
        })
    }

    fn take_ident(&mut self) -> Result<String, BunError<'src>> {
        if let Some(tok) = self.current()
            && let TokenKind::Ident(ident) = tok.clone().value
        {
            self.advance();
            return Ok(ident);
        }

        if let Some(tok) = self.advance() {
            if let TokenKind::Ident(ref v) = tok.value {
                return Ok(v.clone());
            }
        }
        Err(BunError {
            message: "Expected identifier".to_string(),
            span: self.span,
            source: self.code,
        })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.i + 1)
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.i)
    }

    fn advance(&mut self) -> Option<&Token> {
        self.i += 1;
        if let Some(tok) = self.tokens.get(self.i) {
            self.span = tok.span;
            Some(tok)
        } else {
            None
        }
    }
}
