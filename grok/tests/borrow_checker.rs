#[cfg(test)]
mod tests {
    use grok::borrow_checker::{BorrowChecker, BorrowType};
    use grok::ast::Span;

    #[test]
    fn test_borrow_checker_basic() {
        let mut checker = BorrowChecker::new();
        let span = Span { line: 1, col: 1 };
        
        // Multiple immutable borrows OK
        assert!(checker.add_borrow("x".to_string(), BorrowType::Immutable, span.clone()).is_ok());
        assert!(checker.add_borrow("x".to_string(), BorrowType::Immutable, span.clone()).is_ok());
        
        // Mutable borrow fails if immutable exist
        assert!(checker.add_borrow("x".to_string(), BorrowType::Mutable, span.clone()).is_err());
    }

    #[test]
    fn test_borrow_checker_exclusive() {
        let mut checker = BorrowChecker::new();
        let span = Span { line: 1, col: 1 };
        
        // Mutable borrow OK
        assert!(checker.add_borrow("y".to_string(), BorrowType::Mutable, span.clone()).is_ok());
        
        // Any other borrow fails
        assert!(checker.add_borrow("y".to_string(), BorrowType::Immutable, span.clone()).is_err());
        assert!(checker.add_borrow("y".to_string(), BorrowType::Mutable, span.clone()).is_err());
    }

    #[test]
    fn test_borrow_checker_ast() {
        use grok::ast::{AstNode, Span};
        let mut checker = BorrowChecker::new();
        let span = Span { line: 1, col: 1 };

        // { let mut x = 5; { let y = &mut x; } let z = &x; }
        let ast = AstNode::Block(vec![
            AstNode::LetStmt {
                name: "x".to_string(),
                mutable: true,
                ty: None,
                expr: Box::new(AstNode::IntLiteral(5, span.clone())),
                span: span.clone(),
            },
            AstNode::Block(vec![
                AstNode::LetStmt {
                    name: "y".to_string(),
                    mutable: false,
                    ty: None,
                    expr: Box::new(AstNode::UnaryOp {
                        op: "&mut".to_string(),
                        operand: Box::new(AstNode::Identifier("x".to_string(), span.clone())),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                }
            ]),
            AstNode::LetStmt {
                name: "z".to_string(),
                mutable: false,
                ty: None,
                expr: Box::new(AstNode::UnaryOp {
                    op: "&".to_string(),
                    operand: Box::new(AstNode::Identifier("x".to_string(), span.clone())),
                    span: span.clone(),
                }),
                span: span.clone(),
            }
        ]);

        assert!(checker.check(&ast).is_ok(), "Borrow checker failed on valid code: {:?}", checker.check(&ast).err());
    }

    #[test]
    fn test_borrow_checker_fail() {
        use grok::ast::{AstNode, Span};
        let mut checker = BorrowChecker::new();
        let span = Span { line: 1, col: 1 };

        // { let mut x = 5; let y = &mut x; let z = &x; }
        let ast = AstNode::Block(vec![
            AstNode::LetStmt {
                name: "x".to_string(),
                mutable: true,
                ty: None,
                expr: Box::new(AstNode::IntLiteral(5, span.clone())),
                span: span.clone(),
            },
            AstNode::LetStmt {
                name: "y".to_string(),
                mutable: false,
                ty: None,
                expr: Box::new(AstNode::UnaryOp {
                    op: "&mut".to_string(),
                    operand: Box::new(AstNode::Identifier("x".to_string(), span.clone())),
                    span: span.clone(),
                }),
                span: span.clone(),
            },
            AstNode::LetStmt {
                name: "z".to_string(),
                mutable: false,
                ty: None,
                expr: Box::new(AstNode::UnaryOp {
                    op: "&".to_string(),
                    operand: Box::new(AstNode::Identifier("x".to_string(), span.clone())),
                    span: span.clone(),
                }),
                span: span.clone(),
            }
        ]);

        assert!(checker.check(&ast).is_err(), "Borrow checker should have failed on overlapping borrows");
    }
}
