use crate::parser::{Expr, ExprKind};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

impl From<(usize, usize)> for Pos {
    fn from((line, column): (usize, usize)) -> Self {
        Pos { line, column }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl From<(Pos, Pos)> for Span {
    fn from(value: (Pos, Pos)) -> Self {
        Span { start: value.0, end: value.1 }
    }
}

impl From<Pos> for Span {
    fn from(value: Pos) -> Self {
        Span { start: value, end: value }
    }
}

#[derive(Debug, Clone)]
pub struct Locatable<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Locatable<T> {
    pub fn new(kind: T, pos: Pos) -> Self {
        Self {
            value: kind,
            span: Span::from(pos),
        }
    }
    pub fn new_span(kind: T, span: Span) -> Self {
        Self { value: kind, span }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}