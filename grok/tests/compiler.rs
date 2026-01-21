// grok/tests/compiler.rs

use grok::ast::{AstNode, Span, Param};
use grok::compiler::Compiler;

#[test]
fn test_compiler_basic() {
    let span = Span { line: 1, col: 1 };
    let ast = AstNode::Program(vec![
        AstNode::FunctionDef {
            name: "main".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(AstNode::BinaryOp {
                left: Box::new(AstNode::IntLiteral(10, span.clone())),
                op: "+".to_string(),
                right: Box::new(AstNode::IntLiteral(32, span.clone())),
                span: span.clone(),
            }),
            decorators: vec![],
            span: span.clone(),
        }
    ]);

    let mut compiler = Compiler::new();
    let code_ptr = compiler.compile_program(&ast).unwrap();
    
    let main_fn: fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
    let result = main_fn();
    
    assert_eq!(result, 42);
}

#[test]
fn test_compiler_params() {
    let span = Span { line: 1, col: 1 };
    let ast = AstNode::Program(vec![
        AstNode::FunctionDef {
            name: "add".to_string(),
            params: vec![
                Param { name: "a".to_string(), ty: None, span: span.clone() },
                Param { name: "b".to_string(), ty: None, span: span.clone() },
            ],
            return_type: None,
            body: Box::new(AstNode::BinaryOp {
                left: Box::new(AstNode::Identifier("a".to_string(), span.clone())),
                op: "+".to_string(),
                right: Box::new(AstNode::Identifier("b".to_string(), span.clone())),
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
               func: Box::new(AstNode::Identifier("add".to_string(), span.clone())),
               args: vec![
                   AstNode::IntLiteral(20, span.clone()),
                   AstNode::IntLiteral(22, span.clone()),
               ],
               span: span.clone(),
            }),
            decorators: vec![],
            span: span.clone(),
        }
    ]);

    let mut compiler = Compiler::new();
    let code_ptr = compiler.compile_program(&ast).unwrap();
    
    let main_fn: fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
    let result = main_fn();
    
    assert_eq!(result, 42);
}

#[test]
fn test_compiler_recursion() {
    let span = Span { line: 0, col: 0 };
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

    let mut compiler = Compiler::new();
    let code_ptr = compiler.compile_program(&ast).unwrap();
    
    let main_fn: fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
    let result = main_fn();
    
    assert_eq!(result, 120);
}
