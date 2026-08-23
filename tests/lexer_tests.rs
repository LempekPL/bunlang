#[cfg(test)]
mod tests {
    use bunlang::lexer::*;
    use super::*;

    fn kinds(code: &str) -> Vec<TokenKind> {
        Lexer::tokenize(code)
            .unwrap()
            .into_iter()
            .map(|t| t.value)
            .collect()
    }

    #[test]
    fn test_single_number() {
        assert_eq!(
            kinds("123"),
            vec![TokenKind::Number(123.0)]
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(
            kinds("foo_bar"),
            vec![TokenKind::Ident("foo_bar".into())]
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            kinds("let if fn"),
            vec![
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Keyword(Keyword::If),
                TokenKind::Keyword(Keyword::Fn),
            ]
        );
    }

    #[test]
    fn test_basic_symbols() {
        assert_eq!(
            kinds("( ) { } + - * / % = == , ; :"),
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Equals,
                TokenKind::EqualsEquals,
                TokenKind::Comma,
                TokenKind::Semicolon,
                TokenKind::Colon,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            kinds("\"hello\""),
            vec![TokenKind::String("hello".into())]
        );
    }

    #[test]
    fn test_expression_line() {
        assert_eq!(
            kinds("let x = 42 + 3.14;"),
            vec![
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Ident("x".into()),
                TokenKind::Equals,
                TokenKind::Number(42.0),
                TokenKind::Plus,
                TokenKind::Number(3.14),
                TokenKind::Semicolon,
            ]
        );
    }

    #[test]
    fn test_comment_line() {
        assert_eq!(
            kinds("let x = 1; // this is a comment"),
            vec![
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Ident("x".into()),
                TokenKind::Equals,
                TokenKind::Number(1.0),
                TokenKind::Semicolon,
            ]
        );
    }
}
