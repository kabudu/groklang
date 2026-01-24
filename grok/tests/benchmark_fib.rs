#[cfg(test)]
mod benchmarks {
    use grok::ast::{AstNode, Param, Span};
    use grok::ir::IRGenerator;
    use grok::vm::{Value, VM};
    use std::time::Instant;

    fn create_fib_ast() -> AstNode {
        let span = Span { line: 1, col: 1 };
        
        // fn fib(n) { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
        AstNode::Program(vec![
            AstNode::FunctionDef {
                name: "fib".to_string(),
                params: vec![Param { name: "n".to_string(), ty: None, span: span.clone() }],
                return_type: None,
                body: Box::new(AstNode::IfExpr {
                    condition: Box::new(AstNode::BinaryOp {
                        left: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                        op: "<".to_string(),
                        right: Box::new(AstNode::IntLiteral(2, span.clone())),
                        span: span.clone(),
                    }),
                    then_body: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                    else_body: Some(Box::new(AstNode::BinaryOp {
                        left: Box::new(AstNode::FunctionCall {
                            func: Box::new(AstNode::Identifier("fib".to_string(), span.clone())),
                            args: vec![AstNode::BinaryOp {
                                left: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                                op: "-".to_string(),
                                right: Box::new(AstNode::IntLiteral(1, span.clone())),
                                span: span.clone(),
                            }],
                            span: span.clone(),
                        }),
                        op: "+".to_string(),
                        right: Box::new(AstNode::FunctionCall {
                            func: Box::new(AstNode::Identifier("fib".to_string(), span.clone())),
                            args: vec![AstNode::BinaryOp {
                                left: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                                op: "-".to_string(),
                                right: Box::new(AstNode::IntLiteral(2, span.clone())),
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
                    func: Box::new(AstNode::Identifier("fib".to_string(), span.clone())),
                    args: vec![AstNode::IntLiteral(30, span.clone())],
                    span: span.clone(),
                }),
                decorators: vec![],
                span: span.clone(),
            },
        ])
    }

    #[tokio::test]
    async fn benchmark_fib_30() {
        let ast = create_fib_ast();
        
        let mut ir_gen = IRGenerator::new();
        let ir = ir_gen.generate(&ast);
        
        let mut vm = VM::new();
        vm.load_program(&ir);
        
        let start = Instant::now();
        let result = vm.execute("main".to_string(), None).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "VM execution failed: {:?}", result.err());
        let val = result.unwrap();
        
        if let Value::Int(v) = val {
            println!("GrokLang VM fib(30) = {}", v);
            println!("GrokLang VM Time: {:.4}s", duration.as_secs_f64());
            assert_eq!(v, 832040);
        } else {
            panic!("Expected Int, got {:?}", val);
        }
    }
}
