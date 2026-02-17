#[cfg(test)]
mod tests {
    use grok::ast::{AstNode, Span, Type};
    use grok::parser::Parser;
    use grok::type_checker::TypeChecker;

    #[test]
    fn test_type_check_inference() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn add(a, b) { let x = a + b; return x }")
            .unwrap();
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
        // The parser expects a complete program, so "fn err() { ... }" is correct.
        let ast = parser
            .parse("fn err() -> () { let x: i32 = true; return; }")
            .unwrap();
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
            },
        ]);

        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "Failed to type check struct: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_match_guard_must_be_bool() {
        use grok::ast::{MatchArm, Pattern};
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![AstNode::FunctionDef {
            name: "check".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(AstNode::MatchExpr {
                scrutinee: Box::new(AstNode::IntLiteral(1, span.clone())),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::IntLiteral(1),
                        guard: Some(AstNode::IntLiteral(42, span.clone())),
                        body: AstNode::IntLiteral(1, span.clone()),
                    },
                    MatchArm {
                        pattern: Pattern::Underscore,
                        guard: None,
                        body: AstNode::IntLiteral(0, span.clone()),
                    },
                ],
                span: span.clone(),
            }),
            decorators: vec![],
            span: span.clone(),
        }]);
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "Type checker should reject non-bool match guard"
        );
    }

    #[test]
    fn test_or_pattern_type_mismatch() {
        use grok::ast::{MatchArm, Pattern};
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![AstNode::FunctionDef {
            name: "check".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(AstNode::MatchExpr {
                scrutinee: Box::new(AstNode::IntLiteral(1, span.clone())),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Or(vec![
                            Pattern::IntLiteral(1),
                            Pattern::BoolLiteral(true),
                        ]),
                        guard: None,
                        body: AstNode::IntLiteral(1, span.clone()),
                    },
                    MatchArm {
                        pattern: Pattern::Underscore,
                        guard: None,
                        body: AstNode::IntLiteral(0, span.clone()),
                    },
                ],
                span: span.clone(),
            }),
            decorators: vec![],
            span: span.clone(),
        }]);

        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "or-pattern branch types should be consistent"
        );
    }

    #[test]
    fn test_logical_ops_require_bool() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { let x = 1 && 2; }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "logical operators must be bool-only");
    }

    #[test]
    fn test_bool_match_non_exhaustive_fails() {
        use grok::ast::{MatchArm, Pattern};
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![AstNode::FunctionDef {
            name: "check".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(AstNode::MatchExpr {
                scrutinee: Box::new(AstNode::BoolLiteral(true, span.clone())),
                arms: vec![MatchArm {
                    pattern: Pattern::BoolLiteral(true),
                    guard: None,
                    body: AstNode::IntLiteral(1, span.clone()),
                }],
                span: span.clone(),
            }),
            decorators: vec![],
            span: span.clone(),
        }]);
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "non-exhaustive bool match should be rejected"
        );
    }

    #[test]
    fn test_bool_match_exhaustive_passes() {
        use grok::ast::{MatchArm, Pattern};
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![AstNode::FunctionDef {
            name: "check".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(AstNode::MatchExpr {
                scrutinee: Box::new(AstNode::BoolLiteral(true, span.clone())),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::BoolLiteral(true),
                        guard: None,
                        body: AstNode::IntLiteral(1, span.clone()),
                    },
                    MatchArm {
                        pattern: Pattern::BoolLiteral(false),
                        guard: None,
                        body: AstNode::IntLiteral(0, span.clone()),
                    },
                ],
                span: span.clone(),
            }),
            decorators: vec![],
            span: span.clone(),
        }]);
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "exhaustive bool match should type-check");
    }

    #[test]
    fn test_enum_match_non_exhaustive_fails() {
        use grok::ast::{MatchArm, Pattern};
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![
            AstNode::EnumDef {
                name: "Color".to_string(),
                variants: vec![
                    ("Red".to_string(), None),
                    ("Green".to_string(), None),
                    ("Blue".to_string(), None),
                ],
                generics: vec![],
                span: span.clone(),
            },
            AstNode::FunctionDef {
                name: "check".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(AstNode::MatchExpr {
                    scrutinee: Box::new(AstNode::Identifier("Color".to_string(), span.clone())),
                    arms: vec![MatchArm {
                        pattern: Pattern::Enum("Color".to_string(), "Red".to_string(), None),
                        guard: None,
                        body: AstNode::IntLiteral(1, span.clone()),
                    }],
                    span: span.clone(),
                }),
                decorators: vec![],
                span: span.clone(),
            },
        ]);
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "non-exhaustive enum match should be rejected"
        );
    }

    #[test]
    fn test_unknown_generic_constructor_fails() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { let x: Foo<i32> = 1; }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "unknown generic constructor should fail type checking"
        );
    }

    #[test]
    fn test_builtin_generic_arity_mismatch_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn main() { let x: Vec<i32, i32> = 1; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "builtin generic arity mismatch should fail type checking"
        );
    }

    #[test]
    fn test_unknown_trait_type_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn use_it(x: trait Missing) { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "unknown trait type should fail");
    }

    #[test]
    fn test_known_trait_type_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Drawable { fn draw() {} } fn use_it(x: trait Drawable) { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "known trait type should pass");
    }

    #[test]
    fn test_duplicate_trait_definition_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait T { fn a() {} } trait T { fn b() {} }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "duplicate trait definitions should fail");
    }

    #[test]
    fn test_trait_unknown_bound_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Drawable: Missing { fn draw() {} }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "unknown trait bound should fail");
    }

    #[test]
    fn test_trait_self_bound_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Drawable: Drawable { fn draw() {} }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "self trait bound should fail");
    }

    #[test]
    fn test_trait_known_bounds_pass() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Renderable { fn render() {} } trait Drawable: Renderable { fn draw() {} }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "known trait bounds should pass");
    }

    #[test]
    fn test_trait_impl_missing_method_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Drawable { fn draw() {} fn area() {} } struct Point { x: i32 } impl Drawable for Point { fn draw() {} }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "missing trait method in impl should fail");
    }

    #[test]
    fn test_trait_impl_complete_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Drawable { fn draw() {} fn area() {} } struct Point { x: i32 } impl Drawable for Point { fn draw() {} fn area() {} }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "complete trait impl should pass");
    }

    #[test]
    fn test_trait_impl_arity_mismatch_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Drawable { fn draw(x: i32) {} } struct Point { x: i32 } impl Drawable for Point { fn draw() {} }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "trait impl arity mismatch should fail");
    }

    #[test]
    fn test_trait_impl_param_type_mismatch_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Drawable { fn draw(x: i32) {} } struct Point { x: i32 } impl Drawable for Point { fn draw(x: bool) {} }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "trait impl param type mismatch should fail"
        );
    }

    #[test]
    fn test_trait_impl_return_type_mismatch_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Drawable { fn area() -> i32 {} } struct Point { x: i32 } impl Drawable for Point { fn area() -> bool {} }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "trait impl return type mismatch should fail"
        );
    }

    #[test]
    fn test_impl_unknown_trait_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("struct Point { x: i32 } impl Missing for Point { fn draw() {} }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "impl with unknown trait should fail");
    }

    #[test]
    fn test_impl_unknown_type_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Drawable { fn draw() {} } impl Drawable for Missing { fn draw() {} }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "impl for unknown type should fail");
    }

    #[test]
    fn test_type_error_contains_line_col() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { y; }").unwrap();
        let mut checker = TypeChecker::new();
        let err = checker.check(&ast).expect_err("should fail");
        assert!(
            err.contains("line "),
            "error should include line info: {}",
            err
        );
        assert!(
            err.contains(" col "),
            "error should include col info: {}",
            err
        );
    }
}
