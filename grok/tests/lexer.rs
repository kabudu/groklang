#[cfg(test)]
mod tests {
    use grok::lexer::{Lexer, Token};

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("fn let if else");
        assert_eq!(lexer.next(), Some(Ok(Token::Fn)));
        assert_eq!(lexer.next(), Some(Ok(Token::Let)));
        assert_eq!(lexer.next(), Some(Ok(Token::If)));
        assert_eq!(lexer.next(), Some(Ok(Token::Else)));
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = Lexer::new("hello world");
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier)));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier)));
    }

    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new("42 3.14");
        assert_eq!(lexer.next(), Some(Ok(Token::Int)));
        assert_eq!(lexer.next(), Some(Ok(Token::Float)));
    }

    #[test]
    fn test_lexer_strings() {
        let mut lexer = Lexer::new("\"hello\"");
        assert_eq!(lexer.next(), Some(Ok(Token::String)));
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("+ - * /");
        assert_eq!(lexer.next(), Some(Ok(Token::Plus)));
        assert_eq!(lexer.next(), Some(Ok(Token::Minus)));
        assert_eq!(lexer.next(), Some(Ok(Token::Star)));
        assert_eq!(lexer.next(), Some(Ok(Token::Slash)));
    }
}
