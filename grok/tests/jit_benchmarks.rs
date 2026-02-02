#[cfg(test)]
mod jit_benchmarks {
    use grok::ast::{AstNode, Param, Span};
    use grok::ir::IRGenerator;
    use grok::vm::Value;
    use grok::optimizations::OptimizedVM;

    fn create_add_ast() -> AstNode {
        let span = Span { line: 1, col: 1 };
        
        AstNode::Program(vec![
            AstNode::FunctionDef {
                name: "complex_math".to_string(),
                params: vec![
                    Param { name: "a".to_string(), ty: None, span: span.clone() },
                    Param { name: "b".to_string(), ty: None, span: span.clone() }
                ],
                return_type: None,
                body: Box::new(AstNode::BinaryOp {
                    left: Box::new(AstNode::BinaryOp {
                        left: Box::new(AstNode::BinaryOp {
                            left: Box::new(AstNode::Identifier("a".to_string(), span.clone())),
                            op: "+".to_string(),
                            right: Box::new(AstNode::Identifier("b".to_string(), span.clone())),
                            span: span.clone(),
                        }),
                        op: "+".to_string(),
                        right: Box::new(AstNode::Identifier("a".to_string(), span.clone())),
                        span: span.clone(),
                    }),
                    op: "+".to_string(),
                    right: Box::new(AstNode::Identifier("b".to_string(), span.clone())),
                    span: span.clone(),
                }),
                decorators: vec![],
                span: span.clone(),
            },
        ])
    }

    #[test]
    fn benchmark_jit_speed_vs_interpreter() {
        let ast = create_add_ast();
        let mut ir_gen = IRGenerator::new();
        let ir = ir_gen.generate(&ast);
        
        let mut vm = OptimizedVM::new();
        vm.load_program(&ir);
        
        // Trigger JIT
        for i in 0..20 {
            let _ = vm.execute("complex_math", vec![Value::Int(i), Value::Int(1)]);
        }
        
        let iterations = 1_000_000;
        let start_jit = std::time::Instant::now();
        for i in 0..iterations {
            let _ = vm.execute("complex_math", vec![Value::Int(i), Value::Int(1)]);
        }
        let duration_jit = start_jit.elapsed();
        
        let mut vm_no_jit = OptimizedVM::new();
        vm_no_jit.load_program(&ir);
        
        let start_interp = std::time::Instant::now();
        for i in 0..iterations {
            let _ = vm_no_jit.execute("complex_math", vec![Value::Int(i), Value::Int(1)]);
        }
        let duration_interp = start_interp.elapsed();
        
        println!("JIT Complex Math Time: {:?}", duration_jit);
        println!("Interpreter Complex Math Time: {:?}", duration_interp);
    }

    fn create_loop_ast() -> AstNode {
        let span = Span { line: 1, col: 1 };
        
        AstNode::Program(vec![
            AstNode::FunctionDef {
                name: "iter_sum".to_string(),
                params: vec![Param { name: "n".to_string(), ty: None, span: span.clone() }],
                return_type: None,
                body: Box::new(AstNode::Block(vec![
                    AstNode::LetStmt {
                        name: "i".to_string(),
                        mutable: true,
                        ty: None,
                        expr: Box::new(AstNode::IntLiteral(0, span.clone())),
                        span: span.clone()
                    },
                    AstNode::LetStmt {
                        name: "sum".to_string(),
                        mutable: true,
                        ty: None,
                        expr: Box::new(AstNode::IntLiteral(0, span.clone())),
                        span: span.clone()
                    },
                    AstNode::WhileLoop {
                        condition: Box::new(AstNode::BinaryOp {
                            left: Box::new(AstNode::Identifier("i".to_string(), span.clone())),
                            op: "<".to_string(),
                            right: Box::new(AstNode::Identifier("n".to_string(), span.clone())),
                            span: span.clone(),
                        }),
                        body: Box::new(AstNode::Block(vec![
                            AstNode::LetStmt {
                                name: "sum".to_string(),
                                mutable: true,
                                ty: None,
                                expr: Box::new(AstNode::BinaryOp {
                                    left: Box::new(AstNode::Identifier("sum".to_string(), span.clone())),
                                    op: "+".to_string(),
                                    right: Box::new(AstNode::Identifier("i".to_string(), span.clone())),
                                    span: span.clone(),
                                }),
                                span: span.clone(),
                            },
                            AstNode::LetStmt {
                                name: "i".to_string(),
                                mutable: true,
                                ty: None,
                                expr: Box::new(AstNode::BinaryOp {
                                    left: Box::new(AstNode::Identifier("i".to_string(), span.clone())),
                                    op: "+".to_string(),
                                    right: Box::new(AstNode::IntLiteral(1, span.clone())),
                                    span: span.clone(),
                                }),
                                span: span.clone(),
                            },
                        ])),
                        span: span.clone(),
                    },
                    AstNode::Return { value: Some(Box::new(AstNode::Identifier("sum".to_string(), span.clone()))), span: span.clone() },
                ])),
                decorators: vec![],
                span: span.clone(),
            },
        ])
    }

    #[test]
    fn benchmark_jit_loop_performance() {
        let ast = create_loop_ast();
        let mut ir_gen = IRGenerator::new();
        let ir = ir_gen.generate(&ast);
        
        let mut vm = OptimizedVM::new();
        vm.load_program(&ir);
        
        // Trigger JIT
        for _ in 0..20 {
            let _ = vm.execute("iter_sum", vec![Value::Int(10)]);
        }
        
        let large_n = 1_000_000; // Smaller n for faster test if needed, but JIT should handle 10M easily
        let start_jit = std::time::Instant::now();
        let res_jit = vm.execute("iter_sum", vec![Value::Int(large_n)]);
        let duration_jit = start_jit.elapsed();
        
        let mut vm_no_jit = OptimizedVM::new();
        vm_no_jit.load_program(&ir);
        
        let start_interp = std::time::Instant::now();
        let res_interp = vm_no_jit.execute("iter_sum", vec![Value::Int(large_n)]);
        let duration_interp = start_interp.elapsed();
        
        println!("Loop Result: {:?}", res_jit);
        println!("JIT Loop Time (n={}): {:?}", large_n, duration_jit);
        println!("Interpreter Loop Time (n={}): {:?}", large_n, duration_interp);
        
        assert_eq!(res_jit.unwrap(), res_interp.unwrap());
        assert!(duration_jit < duration_interp, "JIT loop should be faster");
    }
}
