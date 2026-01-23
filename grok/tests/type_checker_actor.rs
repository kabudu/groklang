#[cfg(test)]
mod tests {
    use grok::parser::Parser;
    use grok::type_checker::TypeChecker;

    #[test]
    fn test_actor_type_checking() {
        let parser = Parser::new();
        let input = "
            actor MyActor {
                receive {
                    msg => msg
                }
            }

            fn main() -> i32 {
                let a = spawn MyActor {};
                a ! 42;
                0
            }
        ";
        let ast = parser.parse(input).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "Type check failed: {:?}", result.err());
    }
}
