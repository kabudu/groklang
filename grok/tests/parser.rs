#[cfg(test)]
mod tests {
    use grok::ast::AstNode;
    use grok::parser::Parser;

    #[test]
    fn test_parse_function() {
        let parser = Parser::new();
        let result = parser.parse("fn add(a: i32, b: i32) -> i32 {}");
        assert!(result.is_ok());
        if let AstNode::Program(nodes) = result.unwrap() {
            assert_eq!(nodes.len(), 1);
        }
    }

    #[test]
    fn test_parse_invalid() {
        let parser = Parser::new();
        let result = parser.parse("invalid syntax {{{");
        assert!(result.is_err());
    }
}
