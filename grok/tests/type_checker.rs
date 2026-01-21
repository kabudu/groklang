#[cfg(test)]
mod tests {
    use grok::ast::{AstNode, Type};
    use grok::type_checker::{TypeChecker, TypeEnv};

    #[test]
    fn test_type_env() {
        let mut env = TypeEnv::new();
        env.insert("x", Type::Int32);
        assert_eq!(env.lookup("x"), Some(&Type::Int32));
        assert_eq!(env.lookup("y"), None);
    }

    #[test]
    fn test_type_check_function() {
        let mut checker = TypeChecker::new();
        let ast = AstNode::Function {
            name: "test".to_string(),
            params: vec![],
            body: Box::new(AstNode::Program(vec![])),
            return_type: None,
        };
        assert!(checker.check(&ast).is_ok());
    }
}
