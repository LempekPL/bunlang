use crate::error::BunError;
use crate::location::{Locatable, Pos, Span};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Const,
    Let,
    Mut,
    If,
    Fn,
    For,
    While,
    Struct,
    Enum,
    Impl,
    In,
    Else,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keyword::Const => write!(f, "const"),
            Keyword::Let => write!(f, "let"),
            Keyword::Mut => write!(f, "mut"),
            Keyword::If => write!(f, "if"),
            Keyword::Fn => write!(f, "fn"),
            Keyword::For => write!(f, "for"),
            Keyword::While => write!(f, "while"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Enum => write!(f, "enum"),
            Keyword::Impl => write!(f, "impl"),
            Keyword::In => write!(f, "in"),
            Keyword::Else => write!(f, "else"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Keyword(Keyword),
    Number(f64),
    String(String),
    Bool(bool),
    EqualsEquals,
    LessEquals,
    GreaterEquals,
    Less,
    Greater,
    NotEquals,
    Not,
    Equals,
    Plus,
    PlusEquals,
    Minus,
    MinusEquals,
    Star,
    StarEquals,
    Slash,
    SlashEquals,
    Percent,
    PercentEquals,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    ColonColon,
    Colon,
    Dot,
    DotDot,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(ident) => write!(f, "{}", ident),
            Self::Keyword(keyword) => write!(f, "{}", keyword),
            Self::Number(num) => write!(f, "{}", num),
            Self::String(string) => write!(f, "{}", string),
            Self::Bool(bool) => write!(f, "{}", bool),
            Self::EqualsEquals => write!(f, "=="),
            Self::Equals => write!(f, "="),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::Semicolon => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::DotDot => write!(f, ".."),
            Self::Colon => write!(f, ":"),
            Self::ColonColon => write!(f, "::"),
            Self::LessEquals => write!(f, "<="),
            Self::GreaterEquals => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::NotEquals => write!(f, "!="),
            Self::Not => write!(f, "!"),
            Self::PlusEquals => write!(f, "+="),
            Self::MinusEquals => write!(f, "-="),
            Self::StarEquals => write!(f, "*="),
            Self::SlashEquals => write!(f, "/="),
            Self::PercentEquals => write!(f, "%="),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::BitAnd => write!(f, "&"),
            Self::BitOr => write!(f, "|"),
            Self::BitXor => write!(f, "^"),
        }
    }
}

pub type Token = Locatable<TokenKind>;

pub struct Lexer<'src> {
    code: &'src str,
    tokens: Vec<Token>,
    pos: Pos,
}

impl<'src> Lexer<'src> {
    pub fn format(code: &'src str) -> Result<String, BunError<'src>> {
        let tokens = Self::tokenize(code)?;
        let mut formatted = String::new();
        let mut nest = 0;
        let mut peektokens = tokens.into_iter().peekable();

        while let Some(token) = peektokens.next() {
            match &token.value {
                TokenKind::LeftParen => {
                    formatted.push_str(&token.value.to_string());
                }
                TokenKind::Ident(_)
                    if peektokens.peek().is_some_and(|v| {
                        v.value == TokenKind::LeftParen || v.value == TokenKind::RightParen
                    }) =>
                {
                    formatted.push_str(&token.value.to_string());
                }
                TokenKind::Ident(_) => {
                    formatted.push_str(&format!("{} ", token.value));
                }
                TokenKind::LeftBrace => {
                    nest += 1;
                    formatted.push_str(&token.value.to_string());
                    if peektokens
                        .peek()
                        .is_some_and(|v| v.value != TokenKind::RightBrace)
                    {
                        formatted.push('\n');
                        formatted.push_str(&"\t".repeat(nest));
                    }
                }
                TokenKind::RightBrace => {
                    nest = nest.saturating_sub(1);
                    formatted.push_str(&token.value.to_string());
                }
                TokenKind::Semicolon => {
                    formatted.pop();
                    formatted.push_str(&token.value.to_string());
                    formatted.push('\n');
                    if !peektokens
                        .peek()
                        .is_some_and(|v| v.value == TokenKind::RightBrace)
                    {
                        formatted.push_str(&"\t".repeat(nest));
                    }
                }
                _ => {
                    formatted.push_str(&format!("{} ", token.value));
                }
            }
        }

        formatted.push('\n');
        Ok(formatted)
    }

    pub fn tokenize(code: &'src str) -> Result<Vec<Token>, BunError<'src>> {
        let mut lexer = Self {
            code,
            tokens: Vec::new(),
            pos: Pos::from((1, 1)),
        };
        lexer.start()?;
        Ok(lexer.tokens)
    }

    pub fn start(&mut self) -> Result<(), BunError<'src>> {
        let mut chars = self.code.chars().peekable();
        'tok: while let Some(ch) = chars.next() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.bump(ch);
                    continue;
                }
                '!' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::NotEquals, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::Not, None)
                    }
                }
                '<' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::LessEquals, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::Less, None)
                    }
                }
                '>' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(
                            TokenKind::GreaterEquals,
                            Some(Span::from((start, self.pos))),
                        );
                    } else {
                        self.push_token(TokenKind::Greater, None)
                    }
                }
                '=' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(
                            TokenKind::EqualsEquals,
                            Some(Span::from((start, self.pos))),
                        );
                    } else {
                        self.push_token(TokenKind::Equals, None)
                    }
                }
                '+' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::PlusEquals, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::Plus, None);
                    }
                }
                '-' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(
                            TokenKind::MinusEquals,
                            Some(Span::from((start, self.pos))),
                        );
                    } else {
                        self.push_token(TokenKind::Minus, None);
                    }
                }
                '*' => {
                    if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::StarEquals, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::Star, None);
                    }
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        while let Some(ch) = chars.next() {
                            if ch == '\n' {
                                self.bump(ch);
                                continue 'tok;
                            }
                        }
                    } else if chars.peek() == Some(&'*') {
                        self.bump(ch);
                        while let Some(ch) = chars.next() {
                            self.bump(ch);
                            if ch == '*' && chars.peek() == Some(&'/') {
                                self.bump(chars.next().unwrap());
                                continue 'tok;
                            }
                        }
                    } else if chars.peek() == Some(&'=') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(
                            TokenKind::SlashEquals,
                            Some(Span::from((start, self.pos))),
                        );
                    } else {
                        self.push_token(TokenKind::Slash, None)
                    }
                }
                '&' => {
                    if chars.peek() == Some(&'&') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::And, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::BitAnd, None);
                    }
                }
                '|' => {
                    if chars.peek() == Some(&'|') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::Or, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::BitOr, None);
                    }
                }
                '.' => {
                    if chars.peek() == Some(&'.') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::DotDot, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::Dot, None);
                    }
                }
                '^' => self.push_token(TokenKind::BitXor, None),
                '%' => self.push_token(TokenKind::Percent, None),
                '(' => self.push_token(TokenKind::LeftParen, None),
                ')' => self.push_token(TokenKind::RightParen, None),
                '{' => self.push_token(TokenKind::LeftBrace, None),
                '}' => self.push_token(TokenKind::RightBrace, None),
                ';' => self.push_token(TokenKind::Semicolon, None),
                ',' => self.push_token(TokenKind::Comma, None),
                ':' => {
                    if chars.peek() == Some(&':') {
                        let start = self.pos;
                        self.bump(chars.next().unwrap());
                        self.push_token(TokenKind::ColonColon, Some(Span::from((start, self.pos))));
                    } else {
                        self.push_token(TokenKind::Colon, None);
                    }
                }
                '0'..='9' => {
                    let start = self.pos;
                    let mut number_str = ch.to_string();
                    let mut dot = false;
                    while let Some(&v) = chars.peek() {
                        if v == '.' {
                            if dot {
                                break;
                            }
                            dot = true;
                            number_str.push(chars.next().unwrap());
                            self.bump('.');
                        } else if v.is_numeric() {
                            number_str.push(chars.next().unwrap());
                            self.bump(v);
                        } else {
                            break;
                        }
                    }
                    let number = number_str.parse::<f64>();
                    if let Ok(number) = number {
                        self.push_token(
                            TokenKind::Number(number),
                            Some(Span::from((start, self.pos))),
                        );
                    } else {
                        return Err(BunError {
                            message: format!("Invalid number literal '{}'", number_str),
                            span: Span::from((start, self.pos)),
                            source: &self.code,
                        });
                    }
                }
                '"' => {
                    let start = self.pos;
                    let mut string = String::new();
                    while let Some(&next) = chars.peek() {
                        if next == '"' {
                            break;
                        }
                        let next_ch = chars.next().unwrap();
                        if next_ch == '\\' {
                            self.bump(next_ch);
                            if let Some(escaped_ch) = chars.next() {
                                match escaped_ch {
                                    'n' => string.push('\n'),
                                    't' => string.push('\t'),
                                    'r' => string.push('\r'),
                                    '\\' => string.push('\\'),
                                    '"' => string.push('"'),
                                    '0' => string.push('\0'),
                                    other => {
                                        return Err(BunError {
                                            message: format!(
                                                "Unknown escape sequence '\\{}'",
                                                other
                                            ),
                                            span: Span::from((start, self.pos)),
                                            source: self.code,
                                        });
                                    }
                                }
                                self.bump(escaped_ch);
                            } else {
                                return Err(BunError {
                                    message: "Unterminated string literal (ends with '\\')".into(),
                                    span: Span::from((start, self.pos)),
                                    source: self.code,
                                });
                            }
                        } else {
                            if next_ch == '\n' {
                                return Err(BunError {
                                    message:
                                        "Unterminated string literal (found unescaped newline)"
                                            .into(),
                                    span: Span::from((start, self.pos)),
                                    source: self.code,
                                });
                            }
                            string.push(next_ch);
                            self.bump(next_ch);
                        }
                    }
                    if chars.next().is_none() {
                        return Err(BunError {
                            message: "Unterminated string literal".into(),
                            span: Span::from((start, self.pos)),
                            source: self.code,
                        });
                    }
                    self.bump('"');
                    self.push_token(
                        TokenKind::String(string),
                        Some(Span::from((start, self.pos))),
                    );
                }
                v if v.is_alphabetic() || v == '_' => {
                    let start = self.pos;
                    let mut identifier = String::from(v);
                    while chars
                        .peek()
                        .is_some_and(|v| v.is_alphanumeric() || *v == '_')
                    {
                        let next_ch = chars.next().unwrap();
                        identifier.push(next_ch);
                        self.bump(next_ch);
                    }
                    self.push_token(
                        match identifier.as_str() {
                            "true" => TokenKind::Bool(true),
                            "false" => TokenKind::Bool(false),
                            "const" => TokenKind::Keyword(Keyword::Const),
                            "let" => TokenKind::Keyword(Keyword::Let),
                            "mut" => TokenKind::Keyword(Keyword::Mut),
                            "if" => TokenKind::Keyword(Keyword::If),
                            "fn" => TokenKind::Keyword(Keyword::Fn),
                            "for" => TokenKind::Keyword(Keyword::For),
                            "while" => TokenKind::Keyword(Keyword::While),
                            "struct" => TokenKind::Keyword(Keyword::Struct),
                            "enum" => TokenKind::Keyword(Keyword::Enum),
                            "impl" => TokenKind::Keyword(Keyword::Impl),
                            "in" => TokenKind::Keyword(Keyword::In),
                            "else" => TokenKind::Keyword(Keyword::Else),
                            _ => TokenKind::Ident(identifier),
                        },
                        Some(Span::from((start, self.pos))),
                    );
                }
                other => {
                    return Err(BunError {
                        message: format!("Unexpected character '{}'", other),
                        span: Span::from(self.pos),
                        source: &self.code,
                    });
                }
            }
            self.bump(ch);
        }
        Ok(())
    }

    fn bump(&mut self, ch: char) {
        if ch == '\n' {
            self.pos.line += 1;
            self.pos.column = 1;
        } else {
            self.pos.column += 1;
        }
    }

    fn push_token(&mut self, token: TokenKind, span: Option<Span>) {
        if let Some(span) = span {
            self.tokens.push(Token::new_span(token, span))
        } else {
            self.tokens.push(Token::new(token, self.pos));
        }
    }
}
