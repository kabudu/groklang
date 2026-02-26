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

            fn main() {
                let a = spawn MyActor {};
                a ! 42;
            }
        ";
        let ast = parser.parse(input).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "Type check failed: {:?}", result.err());
    }

    #[test]
    fn test_actor_send_message_type_enforced_fails() {
        let parser = Parser::new();
        let input = "
            actor BoolActor {
                receive {
                    true => 1,
                    false => 0
                }
            }

            fn main() {
                let a = spawn BoolActor {};
                a ! 42;
            }
        ";
        let ast = parser.parse(input).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "send should fail when message type does not match actor receive pattern type"
        );
    }

    #[test]
    fn test_actor_send_message_type_enforced_passes() {
        let parser = Parser::new();
        let input = "
            actor BoolActor {
                receive {
                    true => 1,
                    false => 0
                }
            }

            fn main() {
                let a = spawn BoolActor {};
                a ! true;
            }
        ";
        let ast = parser.parse(input).unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "send should pass when message type matches actor receive pattern type"
        );
    }
}
