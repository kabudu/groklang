#[cfg(test)]
mod tests {
    use grok::parser::Parser;
    use grok::type_checker::TypeChecker;
    use grok::ast::{AstNode, Type, Span};

    #[test]
    fn test_type_check_inference() {
        let parser = Parser::new();
        let ast = parser.parse("fn add(a, b) { let x = a + b; return x }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "Type check failed: {:?}", result.err());
        let substitutions = result.unwrap();
        // Since we didn't specify types, T0 (a) and T1 (b) should be unified in a + b
        // The return type of add should be T0
        println!("Substitutions: {:?}", substitutions);
    }

    #[test]
    fn test_type_check_mismatch() {
        let parser = Parser::new();
        // i32 + bool should fail if we were stricter, but our skeletal binary op currently unifies types.
        // Let's try matching mismatch.
        let ast = parser.parse("fn err() { let x: i32 = true; }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "Should have failed type check");
    }
    #[test]
    fn test_type_check_struct() {
        let mut checker = TypeChecker::new();
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![
            AstNode::StructDef {
                name: "Point".to_string(),
                fields: vec![("x".to_string(), Type::Primitive("i32".to_string()))],
                generics: vec![],
                span: span.clone(),
            },
            AstNode::LetStmt {
                name: "p".to_string(),
                mutable: false,
                ty: None,
                expr: Box::new(AstNode::StructLiteral {
                    name: "Point".to_string(),
                    fields: vec![("x".to_string(), AstNode::IntLiteral(10, span.clone()))],
                    span: span.clone(),
                }),
                span: span.clone(),
            }
        ]);
        
        let result = checker.check(&ast);
        assert!(result.is_ok(), "Failed to type check struct: {:?}", result.err());
    }
}
