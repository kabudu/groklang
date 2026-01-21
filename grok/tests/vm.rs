#[cfg(test)]
mod tests {
    use grok::ast::{AstNode, Span, Param};
    use grok::ir::IRGenerator;
    use grok::vm::{VM, Value};

    #[test]
    fn test_ir_generation() {
        let mut gen = IRGenerator::new();
        let ast = AstNode::Program(vec![]);
        let ir = gen.generate(&ast);
        assert_eq!(ir.len(), 0); // Empty program
    }

    #[test]
    fn test_vm_execution() {
        let mut vm = VM::new();
        let mut gen = IRGenerator::new();
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::FunctionDef {
            name: "test".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(AstNode::Block(vec![AstNode::IntLiteral(42, span.clone())])),
            decorators: vec![],
            span: span.clone(),
        };
        let ir = gen.generate(&ast);
        vm.load_program(&ir);
        let result = vm.execute("test").unwrap();
        match result {
            Value::Int(v) => assert_eq!(v, 42),
            _ => panic!("Expected int 42"),
        }
    }
    #[test]
    fn test_vm_struct() {
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![
            AstNode::FunctionDef {
                name: "main".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(AstNode::MemberAccess {
                    object: Box::new(AstNode::StructLiteral {
                        name: "Point".to_string(),
                        fields: vec![("x".to_string(), AstNode::IntLiteral(42, span.clone()))],
                        span: span.clone(),
                    }),
                    member: "x".to_string(),
                    span: span.clone(),
                }),
                decorators: vec![],
                span: span.clone(),
            }
        ]);

        let mut ir_gen = IRGenerator::new();
        let ir = ir_gen.generate(&ast);
        let mut vm = VM::new();
        vm.load_program(&ir);
        let result = vm.execute("main");

        assert!(result.is_ok(), "VM execution failed: {:?}", result.err());
        assert_eq!(result.unwrap().into_int().unwrap(), 42);
    }

    #[test]
    fn test_vm_recursion() {
        let span = Span { line: 0, col: 0 };
        // fn fact(n) { if n == 1 { 1 } else { n * fact(n - 1) } }
        let ast = AstNode::Program(vec![
            AstNode::FunctionDef {
                name: "fact".to_string(),
                params: vec![Param { name: "n".to_string(), ty: None, span: span.clone() }],
                return_type: None,
                body: Box::new(AstNode::IfExpr {
                    condition: Box::new(AstNode::BinaryOp {
                        left: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                        op: "==".to_string(),
                        right: Box::new(AstNode::IntLiteral(1, span.clone())),
                        span: span.clone(),
                    }),
                    then_body: Box::new(AstNode::IntLiteral(1, span.clone())),
                    else_body: Some(Box::new(AstNode::BinaryOp {
                        left: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                        op: "*".to_string(),
                        right: Box::new(AstNode::FunctionCall {
                            func: Box::new(AstNode::Identifier("fact".to_string(), span.clone())),
                            args: vec![AstNode::BinaryOp {
                                left: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                                op: "-".to_string(),
                                right: Box::new(AstNode::IntLiteral(1, span.clone())),
                                span: span.clone(),
                            }],
                            span: span.clone(),
                        }),
                        span: span.clone(),
                    })),
                    span: span.clone(),
                }),
                decorators: vec![],
                span: span.clone(),
            },
            AstNode::FunctionDef {
                name: "main".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(AstNode::FunctionCall {
                    func: Box::new(AstNode::Identifier("fact".to_string(), span.clone())),
                    args: vec![AstNode::IntLiteral(5, span.clone())],
                    span: span.clone(),
                }),
                decorators: vec![],
                span: span.clone(),
            }
        ]);

        let mut ir_gen = IRGenerator::new();
        let ir = ir_gen.generate(&ast);
        let mut vm = VM::new();
        vm.load_program(&ir);
        let result = vm.execute("main");

        assert!(result.is_ok(), "VM execution failed: {:?}", result.err());
        assert_eq!(result.unwrap().into_int().unwrap(), 120);
    }
}
