use grok::parser::Parser;
use grok::macro_expander::MacroExpander;
use grok::ast::{AstNode, Span};

#[test]
fn test_macro_expansion() {
    let source = r#"
        macro_rules! identity {
            (x) => { x }
        }
        let a = identity!(42);
    "#;
    let parser = Parser::new();
    let ast = parser.parse(source).unwrap();
    
    let mut expander = MacroExpander::new();
    let expanded = expander.expand(ast);
    
    // The resultant AST should have 'let a = 42;'
    if let AstNode::Program(nodes) = expanded {
        // MacroDef is filtered out, so first node is LetStmt
        if let AstNode::LetStmt { expr, .. } = &nodes[0] {
            // it's wrapped in a block from the macro template
            if let AstNode::Block(stmts) = &**expr {
                if let AstNode::IntLiteral(val, _) = stmts[0] {
                    assert_eq!(val, 42);
                } else {
                    panic!("Expected IntLiteral(42) in block, got {:?}", stmts[0]);
                }
            } else {
                panic!("Expected Block, got {:?}", expr);
            }
        } else {
            panic!("Expected LetStmt, got {:?}", nodes[0]);
        }
    } else {
        panic!("Expected Program, got {:?}", expanded);
    }
}

#[test]
fn test_macro_multiple_rules() {
    let source = r#"
        macro_rules! multi {
            (1) => { 10 }
            (2) => { 20 }
        }
        let x = multi!(1);
        let y = multi!(2);
    "#;
    let parser = Parser::new();
    let ast = parser.parse(source).unwrap();
    if let AstNode::Program(ref nodes) = ast {
        assert_eq!(nodes.len(), 3, "Expected 3 nodes before expansion, got {:?}", nodes);
    }
    
    let mut expander = MacroExpander::new();
    let expanded = expander.expand(ast);
    
    if let AstNode::Program(nodes) = expanded {
        assert_eq!(nodes.len(), 2, "Expected 2 nodes after expansion, got {:?}", nodes);
        // nodes[0] is x = 10
        // nodes[1] is y = 20
        assert_eq!(nodes.len(), 2);
        if let AstNode::LetStmt { expr, .. } = &nodes[0] {
             if let AstNode::Block(s) = &**expr {
                 if let AstNode::IntLiteral(v, _) = s[0] { assert_eq!(v, 10); }
             }
        }
        if let AstNode::LetStmt { expr, .. } = &nodes[1] {
             if let AstNode::Block(s) = &**expr {
                 if let AstNode::IntLiteral(v, _) = s[0] { assert_eq!(v, 20); }
             }
        }
    }
}
