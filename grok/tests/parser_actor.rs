#[cfg(test)]
mod tests {
    use grok::ast::AstNode;
    use grok::parser::Parser;

    #[test]
    fn test_actor_parsing() {
        let parser = Parser::new();
        let input = "
            actor MyActor {
                receive {
                    msg => println!(msg)
                }
            }

            fn main() {
                let a = spawn MyActor {};
                a ! 42;
            }
        ";
        let result = parser.parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }
}
