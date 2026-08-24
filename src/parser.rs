use crate::error::BunError;
use crate::lexer::{Keyword, Token, TokenKind};
use crate::location::{Locatable, Span};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ExprKind {
    Break,
    Continue,
    Return,
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
        path: Vec<String>,
        args: Vec<Expr>,
    },
    Assign {
        op: TokenKind,
        target: Box<Expr>,
        value: Box<Expr>,
    },
    QualifiedIdent(Vec<String>),
    If {
        condition: Box<Expr>,
        truthy: Box<Expr>,
        falsy: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    For {
        item: String,
        iterable: Box<Expr>,
        body: Box<Expr>,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Box<Expr>)>,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
}

pub type Expr = Locatable<ExprKind>;

#[derive(Debug)]
pub enum StmtKind {
    Declare { name: String, expr: Expr },
    Expr(Expr),
}

pub type Stmt = Locatable<StmtKind>;

#[derive(Debug)]
pub enum ItemKind {
    FnDeclare {
        name: String,
        inputs: Vec<String>,
        block: Expr,
    },
    // StructDeclare {
    //     name: String,
    //     fields: Vec<(String, Type)>,
    // },
    // Impl {
    //     target: String,
    //     items: Vec<Item>,
    // },
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
enum Precedence {
    None = 0,
    Assignment, // = += -= *= /= %=
    Range,      // ..
    Or,         // ||
    And,        // &&
    BitOr,      // |
    BitXor,     // ^
    BitAnd,     // &
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * / %
    Unary,      // ! -
    Call,       // . ()
}

impl Precedence {
    pub fn from_token(kind: &TokenKind) -> Self {
        match kind {
            TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::StarEquals
            | TokenKind::SlashEquals
            | TokenKind::PercentEquals => Precedence::Assignment,
            TokenKind::DotDot => Precedence::Range,
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::BitOr => Precedence::BitOr,
            TokenKind::BitXor => Precedence::BitXor,
            TokenKind::BitAnd => Precedence::BitAnd,
            TokenKind::EqualsEquals | TokenKind::NotEquals => Precedence::Equality,
            TokenKind::Less
            | TokenKind::LessEquals
            | TokenKind::Greater
            | TokenKind::GreaterEquals => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::Dot | TokenKind::LeftParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

impl<'src> Parser<'src> {
    pub fn parse(
        tokens: Vec<Token>,
        code: &'src str,
    ) -> Result<HashMap<String, Item>, BunError<'src>> {
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
                    self.next_token();
                    let ident = self.take_ident()?;
                    self.consume(TokenKind::LeftParen)?;
                    let mut inputs = Vec::new();
                    if self.current_inside() != Some(TokenKind::RightParen) {
                        // todo: implement input types
                        loop {
                            inputs.push(self.take_ident()?);
                            if self.current_inside() == Some(TokenKind::RightParen) {
                                break;
                            }
                            self.consume(TokenKind::Comma)?;
                            if self.current_inside() == Some(TokenKind::RightParen) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RightParen)?;
                    let stmts = self.parse_stmts()?;
                    let span = Span::from((start, stmts.span.end));
                    let fun = ItemKind::FnDeclare {
                        name: ident.clone(),
                        inputs,
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
            let stmt_start = tok.span.start;

            match &tok.value {
                TokenKind::Keyword(Keyword::Let) => {
                    self.next_token();
                    let name = self.take_ident()?;
                    self.consume(TokenKind::Equals)?;
                    let expr = self.parse_expr(Precedence::None)?;
                    self.consume(TokenKind::Semicolon)?;
                    let span = Span::from((stmt_start, expr.span.end));
                    let decl = StmtKind::Declare { name, expr };
                    stmts.push(Stmt::new_span(decl, span));
                }
                TokenKind::RightBrace => break,
                _ => {
                    let expr = self.parse_expr(Precedence::None)?;
                    let requires_semi = match &expr.value {
                        ExprKind::If { .. }
                        | ExprKind::While { .. }
                        | ExprKind::For { .. }
                        | ExprKind::Block { .. } => false,
                        _ => true,
                    };
                    if requires_semi {
                        self.consume(TokenKind::Semicolon)?;
                    }
                    let span = Span::from((stmt_start, self.span.end));
                    stmts.push(Stmt::new_span(StmtKind::Expr(expr), span));
                }
            }
        }

        self.consume(TokenKind::RightBrace)?;
        Ok(Expr::new_span(
            ExprKind::Block { stmts },
            Span::from((start, self.span.end)),
        ))
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Result<Expr, BunError<'src>> {
        let mut left = self.parse_prefix()?;

        while let Some(tok) = self.current() {
            let next_precedence = Precedence::from_token(&tok.value);

            if precedence >= next_precedence {
                break;
            }
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, BunError<'src>> {
        let tok = self.next_token().cloned().ok_or_else(|| BunError {
            message: "Unexpected EOF".into(),
            span: self.span,
            source: self.code,
        })?;

        match &tok.value {
            TokenKind::Keyword(Keyword::Break) => Ok(Expr::new_span(ExprKind::Break, tok.span)),
            TokenKind::Keyword(Keyword::Continue) => {
                Ok(Expr::new_span(ExprKind::Continue, tok.span))
            }
            TokenKind::Number(_) | TokenKind::String(_) => Ok(Expr::new_span(
                ExprKind::Literal(tok.value.clone()),
                tok.span,
            )),
            TokenKind::Ident(name) => {
                let mut path = vec![name.clone()];
                while self
                    .current()
                    .is_some_and(|t| t.value == TokenKind::ColonColon)
                {
                    self.next_token();
                    path.push(self.take_ident()?);
                }
                if path.len() == 1 {
                    Ok(Expr::new_span(ExprKind::Ident(name.clone()), tok.span))
                } else {
                    Ok(Expr::new_span(ExprKind::QualifiedIdent(path), tok.span))
                }
            }
            TokenKind::Minus => {
                let expr = self.parse_expr(Precedence::Unary)?;
                let span = Span::from((tok.span.start, expr.span.end));
                Ok(Expr::new_span(
                    ExprKind::Unary {
                        op: TokenKind::Minus,
                        expr: Box::new(expr),
                    },
                    span,
                ))
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expr(Precedence::None)?;
                self.consume(TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::Keyword(Keyword::If) => {
                let start = tok.span.start;
                let condition = self.parse_expr(Precedence::None)?;
                let truthy = self.parse_stmts()?;
                let mut falsy = None;
                if self
                    .current()
                    .is_some_and(|t| t.value == TokenKind::Keyword(Keyword::Else))
                {
                    self.next_token();
                    if self
                        .current()
                        .is_some_and(|t| t.value == TokenKind::Keyword(Keyword::If))
                    {
                        falsy = Some(Box::new(self.parse_expr(Precedence::None)?));
                    } else {
                        falsy = Some(Box::new(self.parse_stmts()?));
                    }
                }

                let span = Span::from((start, self.span.end));
                Ok(Expr::new_span(
                    ExprKind::If {
                        condition: Box::new(condition),
                        truthy: Box::new(truthy),
                        falsy,
                    },
                    span,
                ))
            }
            TokenKind::Keyword(Keyword::While) => {
                let start = tok.span.start;
                let condition = self.parse_expr(Precedence::None)?;
                let body = self.parse_stmts()?;
                let span = Span::from((start, self.span.end));
                Ok(Expr::new_span(
                    ExprKind::While {
                        condition: Box::new(condition),
                        body: Box::new(body),
                    },
                    span,
                ))
            }
            TokenKind::Keyword(Keyword::For) => {
                let start = tok.span.start;
                let item = self.take_ident()?;
                self.consume(TokenKind::Keyword(Keyword::In))?;
                let iterable = self.parse_expr(Precedence::None)?;
                let body = self.parse_stmts()?;
                let span = Span::from((start, self.span.end));
                Ok(Expr::new_span(
                    ExprKind::For {
                        item,
                        iterable: Box::new(iterable),
                        body: Box::new(body),
                    },
                    span,
                ))
            }
            other => Err(BunError {
                message: format!("Expected expression, found '{}'", other),
                span: tok.span,
                source: self.code,
            }),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr, BunError<'src>> {
        let op_tok = self.next_token().cloned().ok_or_else(|| BunError {
            message: "Unexpected EOF".into(),
            span: self.span,
            source: self.code,
        })?;

        match &op_tok.value {
            TokenKind::LeftParen => {
                let mut args = Vec::new();
                while let Some(arg_tok) = self.current() {
                    if arg_tok.value == TokenKind::RightParen {
                        break;
                    }
                    args.push(self.parse_expr(Precedence::None)?);
                    if self.current().is_some_and(|t| t.value == TokenKind::Comma) {
                        self.next_token();
                    }
                }
                self.consume(TokenKind::RightParen)?;
                let span = Span::from((left.span.start, self.span.end));
                match left.value {
                    ExprKind::QualifiedIdent(path) => {
                        Ok(Expr::new_span(ExprKind::Call { path, args }, span))
                    }
                    ExprKind::Ident(name) => Ok(Expr::new_span(
                        ExprKind::Call {
                            path: vec![name],
                            args,
                        },
                        span,
                    )),
                    _ => Err(BunError {
                        message: "Invalid function call target".to_string(),
                        span: left.span,
                        source: self.code,
                    }),
                }
            }
            TokenKind::Dot => {
                let field_or_method = self.take_ident()?;
                if self
                    .current()
                    .is_some_and(|t| t.value == TokenKind::LeftParen)
                {
                    self.next_token();
                    let mut args = Vec::new();
                    while let Some(arg_tok) = self.current() {
                        if arg_tok.value == TokenKind::RightParen {
                            break;
                        }
                        args.push(self.parse_expr(Precedence::None)?);
                        if self.current().is_some_and(|t| t.value == TokenKind::Comma) {
                            self.next_token();
                        }
                    }
                    self.consume(TokenKind::RightParen)?;
                    let span = Span::from((left.span.start, self.span.end));

                    Ok(Expr::new_span(
                        ExprKind::MethodCall {
                            receiver: Box::new(left),
                            method: field_or_method,
                            args,
                        },
                        span,
                    ))
                } else {
                    let span = Span::from((left.span.start, self.span.end));
                    Ok(Expr::new_span(
                        ExprKind::FieldAccess {
                            expr: Box::new(left),
                            field: field_or_method,
                        },
                        span,
                    ))
                }
            }
            TokenKind::DotDot => {
                let precedence = Precedence::from_token(&op_tok.value);
                let right = self.parse_expr(precedence)?;
                let span = Span::from((left.span.start, right.span.end));
                Ok(Expr::new_span(
                    ExprKind::Binary {
                        op: op_tok.value.clone(),
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                ))
            }
            TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::StarEquals
            | TokenKind::SlashEquals
            | TokenKind::PercentEquals => {
                let right = self.parse_expr(Precedence::Assignment)?;
                let span = Span::from((left.span.start, right.span.end));

                match left.value {
                    ExprKind::Ident(_) | ExprKind::FieldAccess { .. } => Ok(Expr::new_span(
                        ExprKind::Assign {
                            op: op_tok.value.clone(),
                            target: Box::new(left),
                            value: Box::new(right),
                        },
                        span,
                    )),
                    _ => Err(BunError {
                        message: "Invalid assignment target".to_string(),
                        span: left.span,
                        source: self.code,
                    }),
                }
            }
            op => {
                let precedence = Precedence::from_token(op);
                let right = self.parse_expr(precedence)?;
                let span = Span::from((left.span.start, right.span.end));
                Ok(Expr::new_span(
                    ExprKind::Binary {
                        op: op.clone(),
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                ))
            }
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<(), BunError<'src>> {
        if self.current().is_some_and(|v| v.value == kind) {
            self.next_token();
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
            self.next_token();
            return Ok(ident);
        }

        if let Some(tok) = self.next_token() {
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

    fn peek_inside(&self) -> Option<TokenKind> {
        self.tokens.get(self.i + 1).map(|tok| tok.value.clone())
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.i)
    }
    fn current_inside(&self) -> Option<TokenKind> {
        self.tokens.get(self.i).map(|tok| tok.value.clone())
    }

    fn next_token(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.i)?;
        self.span = tok.span;
        self.i += 1;
        Some(tok)
    }
}
