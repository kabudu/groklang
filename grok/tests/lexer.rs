#[cfg(test)]
mod tests {
    use grok::lexer::{Lexer, Token};

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("fn let if else");
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Fn);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Let);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::If);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Else);
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = Lexer::new("hello world");
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Identifier);
    }

    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new("42 3.14");
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Int);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Float);
    }

    #[test]
    fn test_lexer_strings() {
        let mut lexer = Lexer::new("\"hello\"");
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::String);
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("+ - * /");
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Plus);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Minus);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Star);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Slash);
    }
}
