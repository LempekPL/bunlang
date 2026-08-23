use std::fmt;
use crate::location::Span;

#[derive(Debug, Clone)]
pub struct BunError<'src> {
    pub message: String,
    pub span: Span,
    pub source: &'src str,
}

impl<'src> fmt::Display for BunError<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Bun error at line {}, column {}: {}",
            self.span.start.line, self.span.start.column, self.message
        )?;

        if let Some(line_str) = self.source.lines().nth(self.span.start.line - 1) {
            writeln!(f, "{}", line_str)?;
            let start_col = self.span.start.column.saturating_sub(1);
            let end_col = self.span.end.column.max(start_col + 1);

            let mut indicator = String::new();
            for i in 0..end_col {
                indicator.push(if i >= start_col { '^' } else { ' ' });
            }
            writeln!(f, "{}", indicator)?;
        }
        Ok(())
    }
}

impl<'src> std::error::Error for BunError<'src> {}