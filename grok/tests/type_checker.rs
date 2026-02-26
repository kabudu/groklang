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
    fn test_char_literal_type_checks() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { let c: char = 'a'; }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "char literal should type-check");
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
    fn test_struct_literal_missing_field_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("struct Point { x: i32, y: i32 } fn main() { let p = Point { x: 1 }; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "missing struct field should fail");
    }

    #[test]
    fn test_struct_literal_unknown_field_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("struct Point { x: i32 } fn main() { let p = Point { x: 1, y: 2 }; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "unknown struct field should fail");
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
    fn test_index_access_vec_typechecks() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn get(v: Vec<i32>) -> i32 { v[0] }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "indexing Vec<i32> should type-check");
    }

    #[test]
    fn test_index_access_requires_int_index() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn get(v: Vec<i32>) -> i32 { v[true] }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "index access should require i32 index");
    }

    #[test]
    fn test_try_operator_result_typechecks() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn get(v: Result<i32, i32>) -> i32 { v? }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "try operator should unwrap Result<T, E> to T");
    }

    #[test]
    fn test_try_operator_non_tryable_fails() {
        let parser = Parser::new();
        let ast = parser.parse("fn get(v: i32) -> i32 { v? }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "try operator on non-tryable type should fail");
    }

    #[test]
    fn test_array_literal_uniform_type_checks() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn main() { let xs: Vec<i32> = [1, 2, 3]; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "uniform array literal should type-check");
    }

    #[test]
    fn test_array_literal_mixed_types_fail() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { let xs = [1, true]; }").unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "mixed-type array literal should fail");
    }

    #[test]
    fn test_closure_expression_type_checks() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn main() { let f = |x: i32| x + 1; let y: i32 = f(2); }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "closure expression should type-check");
    }

    #[test]
    fn test_macro_call_known_macro_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse("macro_rules! m { (x) => { x } } fn main() { m!(1); }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "known macro call should pass");
    }

    #[test]
    fn test_macro_call_unknown_macro_fails_with_position() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { nope!(1); }").unwrap();
        let mut checker = TypeChecker::new();
        let err = checker
            .check(&ast)
            .expect_err("unknown macro call should fail");
        assert!(
            err.contains("Unknown macro") && err.contains("line") && err.contains("col"),
            "unknown macro error should include line/col, got: {}",
            err
        );
    }

    #[test]
    fn test_macro_expansion_participates_in_type_checking_pass() {
        let parser = Parser::new();
        let ast = parser
            .parse("macro_rules! id { (x) => { x } } fn main() { let b: bool = id!(true); }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "expanded macro expression should participate in type checking"
        );
    }

    #[test]
    fn test_macro_expansion_participates_in_type_checking_fail() {
        let parser = Parser::new();
        let ast = parser
            .parse("macro_rules! id { (x) => { x } } fn main() { let b: bool = id!(1); }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "expanded macro expression should still fail mismatched type checks"
        );
    }

    #[test]
    fn test_guarded_bool_match_not_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn main(b: bool) { match b { true if b => 1, false => 0 } }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "guarded bool arm should not count as exhaustive coverage"
        );
    }

    #[test]
    fn test_guarded_enum_match_not_exhaustive_fails() {
        use grok::ast::{MatchArm, Pattern};
        let span = Span { line: 1, col: 1 };
        let ast = AstNode::Program(vec![
            AstNode::EnumDef {
                name: "Color".to_string(),
                variants: vec![("Red".to_string(), None), ("Blue".to_string(), None)],
                generics: vec![],
                span: span.clone(),
            },
            AstNode::FunctionDef {
                name: "main".to_string(),
                params: vec![],
                return_type: None,
                body: Box::new(AstNode::MatchExpr {
                    scrutinee: Box::new(AstNode::Identifier("Color".to_string(), span.clone())),
                    arms: vec![
                        MatchArm {
                            pattern: Pattern::Enum("Color".to_string(), "Red".to_string(), None),
                            guard: Some(AstNode::BoolLiteral(true, span.clone())),
                            body: AstNode::IntLiteral(1, span.clone()),
                        },
                        MatchArm {
                            pattern: Pattern::Enum("Color".to_string(), "Blue".to_string(), None),
                            guard: None,
                            body: AstNode::IntLiteral(0, span.clone()),
                        },
                    ],
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
            "guarded enum arm should not count as exhaustive coverage"
        );
    }

    #[test]
    fn test_tuple_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "fn main() { match (true, false) { (true, true) => 1, (true, false) => 2, (false, true) => 3, (false, false) => 4 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "exhaustive tuple bool match should pass");
    }

    #[test]
    fn test_tuple_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "fn main() { match (true, false) { (true, true) => 1, (true, false) => 2, (false, true) => 3 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "non-exhaustive tuple bool match should fail");
    }

    #[test]
    fn test_struct_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "struct Flags { a: bool, b: bool } fn main() { match Flags { a: true, b: false } { Flags { a: true, b: true } => 1, Flags { a: true, b: false } => 2, Flags { a: false, b: true } => 3, Flags { a: false, b: false } => 4 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "exhaustive struct bool match should pass");
    }

    #[test]
    fn test_struct_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "struct Flags { a: bool, b: bool } fn main() { match Flags { a: true, b: false } { Flags { a: true, b: true } => 1, Flags { a: true, b: false } => 2, Flags { a: false, b: true } => 3 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "non-exhaustive struct bool match should fail");
    }

    #[test]
    fn test_tuple_enum_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Color { Red, Blue } fn main(c: Color) { match (c, true) { (Color::Red, true) => 1, (Color::Red, false) => 2, (Color::Blue, true) => 3, (Color::Blue, false) => 4 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "exhaustive tuple(enum,bool) match should pass"
        );
    }

    #[test]
    fn test_tuple_enum_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Color { Red, Blue } fn main(c: Color) { match (c, true) { (Color::Red, true) => 1, (Color::Red, false) => 2, (Color::Blue, true) => 3 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "non-exhaustive tuple(enum,bool) match should fail"
        );
    }

    #[test]
    fn test_use_internal_symbol_resolution_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse("mod m { fn f() {} } use m::f; fn main() { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "use of existing internal symbol should pass"
        );
    }

    #[test]
    fn test_use_internal_symbol_resolution_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("mod m { fn f() {} } use m::missing; fn main() { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let err = checker
            .check(&ast)
            .expect_err("use of missing internal symbol should fail");
        assert!(
            err.contains("line") && err.contains("col"),
            "use-path error should include line/col, got: {}",
            err
        );
    }

    #[test]
    fn test_use_alias_imported_function_call_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse("mod m { fn f() -> i32 { 1 } } use m::f as g; fn main() -> i32 { g() }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "alias-imported function should be callable via alias name"
        );
    }

    #[test]
    fn test_use_group_alias_imported_functions_pass() {
        let parser = Parser::new();
        let ast = parser
            .parse("mod m { fn f() -> i32 { 1 } fn h() -> i32 { 2 } } use m::{f as g, h}; fn main() -> i32 { g() + h() }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "grouped alias imports should bind function names into type checker globals"
        );
    }

    #[test]
    fn test_where_trait_bound_enforced_at_call_site_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } struct Point { x: i32 } impl Show for Point { fn show() {} } fn print_it(x: T) where T: Show { return; } fn main() { let p = Point { x: 1 }; print_it(p); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "where-bound trait should be satisfied by impl at call site"
        );
    }

    #[test]
    fn test_where_trait_bound_enforced_at_call_site_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } struct Point { x: i32 } fn print_it(x: T) where T: Show { return; } fn main() { let p = Point { x: 1 }; print_it(p); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "where-bound trait should fail when impl is missing at call site"
        );
    }

    #[test]
    fn test_enum_payload_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Flag { On(bool), Off } fn main(x: Flag) { match x { Flag::On(true) => 1, Flag::On(false) => 2, Flag::Off => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "enum payload exhaustive match should pass for finite payload domain"
        );
    }

    #[test]
    fn test_enum_payload_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Flag { On(bool), Off } fn main(x: Flag) { match x { Flag::On(true) => 1, Flag::Off => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "enum payload non-exhaustive match should fail for finite payload domain"
        );
    }

    #[test]
    fn test_polymorphic_function_calls_independent_instantiation() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "fn id(x: T) -> T { x } fn main() { let a: i32 = id(1); let b: bool = id(true); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "polymorphic function should instantiate independently per call"
        );
    }

    #[test]
    fn test_where_trait_bound_enforced_for_generic_constructor_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for Vec { fn show() {} } fn print_it(x: T) where T: Show { return; } fn main() { let v: Vec<i32> = [1, 2]; print_it(v); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "where-bound should accept generic constructor when impl exists"
        );
    }

    #[test]
    fn test_where_trait_bound_enforced_for_generic_constructor_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } fn print_it(x: T) where T: Show { return; } fn main() { let v: Vec<i32> = [1, 2]; print_it(v); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "where-bound should reject generic constructor when impl is missing"
        );
    }

    #[test]
    fn test_where_trait_bound_with_generic_impl_bounds_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for i32 { fn show() {} } impl<T: Show> Show for Vec<T> { fn show() {} } fn print_it(x: T) where T: Show { return; } fn main() { let v: Vec<i32> = [1, 2]; print_it(v); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "generic impl bounds should satisfy where-bound for Vec<i32>"
        );
    }

    #[test]
    fn test_where_trait_bound_with_generic_impl_bounds_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl<T: Show> Show for Vec<T> { fn show() {} } fn print_it(x: T) where T: Show { return; } fn main() { let v: Vec<i32> = [1, 2]; print_it(v); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "generic impl bounds should fail when element bound is unsatisfied"
        );
    }

    #[test]
    fn test_where_trait_bound_with_repeated_impl_type_param_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for i32 { fn show() {} } impl Show for bool { fn show() {} } impl<T: Show> Show for HashMap<T, T> { fn show() {} } fn print_it(x: X) where X: Show { return; } fn ok(p: HashMap<i32, i32>) { print_it(p); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "repeated impl parameter should match when concrete args are equal"
        );
    }

    #[test]
    fn test_where_trait_bound_with_repeated_impl_type_param_fails_on_mismatch() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for i32 { fn show() {} } impl Show for bool { fn show() {} } impl<T: Show> Show for HashMap<T, T> { fn show() {} } fn print_it(x: X) where X: Show { return; } fn bad(p: HashMap<i32, bool>) { print_it(p); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "repeated impl parameter should reject mismatched concrete args"
        );
    }

    #[test]
    fn test_where_trait_bound_with_two_parameter_impl_bounds_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for i32 { fn show() {} } impl Show for bool { fn show() {} } impl<K: Show, V: Show> Show for HashMap<K, V> { fn show() {} } fn print_it(x: X) where X: Show { return; } fn ok(m: HashMap<i32, bool>) { print_it(m); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "two-parameter generic impl bounds should satisfy where-bound when both args satisfy bounds"
        );
    }

    #[test]
    fn test_where_trait_bound_with_two_parameter_impl_bounds_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for i32 { fn show() {} } impl<K: Show, V: Show> Show for HashMap<K, V> { fn show() {} } fn print_it(x: X) where X: Show { return; } fn bad(m: HashMap<i32, bool>) { print_it(m); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "two-parameter generic impl bounds should fail when one arg misses bound"
        );
    }

    #[test]
    fn test_where_trait_bound_satisfied_via_subtrait_impl_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Base { fn b() {} } trait Sub: Base { fn s() {} } impl Sub for i32 { fn s() {} } fn needs_base(x: T) where T: Base { return; } fn main() { let x: i32 = 1; needs_base(x); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "where-bound should be satisfied transitively via subtrait impl"
        );
    }

    #[test]
    fn test_where_trait_bound_not_satisfied_without_subtrait_relation_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Base { fn b() {} } trait Sub { fn s() {} } impl Sub for i32 { fn s() {} } fn needs_base(x: T) where T: Base { return; } fn main() { let x: i32 = 1; needs_base(x); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "where-bound should fail when subtrait does not imply required trait"
        );
    }

    #[test]
    fn test_where_bound_unresolved_type_var_in_return_position_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } fn make() -> T where T: Show { 1 } fn main() { make(); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "where-bound should fail when bound type variable cannot be resolved from call site"
        );
    }

    #[test]
    fn test_where_bound_partially_unresolved_type_var_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl Show for i32 { fn show() {} } fn make(x: T) -> U where T: Show, U: Show { x } fn main() { make(1); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "where-bound should fail when some bound variables remain unresolved at call site"
        );
    }

    #[test]
    fn test_generic_impl_subtrait_satisfies_supertrait_where_bound_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Base { fn b() {} } trait Sub: Base { fn s() {} } impl Sub for i32 { fn s() {} } impl<T: Base> Sub for Vec<T> { fn s() {} } fn needs_base(x: X) where X: Base { return; } fn main() { let xs: Vec<i32> = [1, 2]; needs_base(xs); }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "generic subtrait impl should satisfy supertrait where-bound transitively"
        );
    }

    #[test]
    fn test_enum_payload_tuple_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum PairWrap { Pair((bool, bool)), Unit } fn main(x: PairWrap) { match x { PairWrap::Pair((true, true)) => 1, PairWrap::Pair((true, false)) => 2, PairWrap::Pair((false, true)) => 3, PairWrap::Pair((false, false)) => 4, PairWrap::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "tuple-payload enum exhaustive match should pass"
        );
    }

    #[test]
    fn test_enum_payload_tuple_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum PairWrap { Pair((bool, bool)), Unit } fn main(x: PairWrap) { match x { PairWrap::Pair((true, true)) => 1, PairWrap::Pair((true, false)) => 2, PairWrap::Pair((false, true)) => 3, PairWrap::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "tuple-payload enum non-exhaustive match should fail"
        );
    }

    #[test]
    fn test_enum_payload_struct_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "struct Flags { a: bool, b: bool } enum Wrap { Flags(Flags), Unit } fn main(x: Wrap) { match x { Wrap::Flags(Flags { a: true, b: true }) => 1, Wrap::Flags(Flags { a: true, b: false }) => 2, Wrap::Flags(Flags { a: false, b: true }) => 3, Wrap::Flags(Flags { a: false, b: false }) => 4, Wrap::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "struct-payload enum exhaustive match should pass"
        );
    }

    #[test]
    fn test_enum_payload_struct_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "struct Flags { a: bool, b: bool } enum Wrap { Flags(Flags), Unit } fn main(x: Wrap) { match x { Wrap::Flags(Flags { a: true, b: true }) => 1, Wrap::Flags(Flags { a: true, b: false }) => 2, Wrap::Flags(Flags { a: false, b: true }) => 3, Wrap::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "struct-payload enum non-exhaustive match should fail"
        );
    }

    #[test]
    fn test_enum_payload_nested_enum_bool_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Inner { A(bool), B } enum Outer { Wrap(Inner), Unit } fn main(x: Outer) { match x { Outer::Wrap(Inner::A(true)) => 1, Outer::Wrap(Inner::A(false)) => 2, Outer::Wrap(Inner::B) => 3, Outer::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "nested enum-payload exhaustive match should pass"
        );
    }

    #[test]
    fn test_enum_payload_nested_enum_bool_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Inner { A(bool), B } enum Outer { Wrap(Inner), Unit } fn main(x: Outer) { match x { Outer::Wrap(Inner::A(true)) => 1, Outer::Wrap(Inner::B) => 3, Outer::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "nested enum-payload non-exhaustive match should fail"
        );
    }

    #[test]
    fn test_enum_payload_nested_struct_enum_match_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Inner { A(bool), B } struct Boxed { inner: Inner, flag: bool } enum Outer { Wrap(Boxed), Unit } fn main(x: Outer) { match x { Outer::Wrap(Boxed { inner: Inner::A(true), flag: true }) => 1, Outer::Wrap(Boxed { inner: Inner::A(true), flag: false }) => 2, Outer::Wrap(Boxed { inner: Inner::A(false), flag: true }) => 3, Outer::Wrap(Boxed { inner: Inner::A(false), flag: false }) => 4, Outer::Wrap(Boxed { inner: Inner::B, flag: true }) => 5, Outer::Wrap(Boxed { inner: Inner::B, flag: false }) => 6, Outer::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "nested struct+enum payload exhaustive match should pass"
        );
    }

    #[test]
    fn test_enum_payload_nested_struct_enum_match_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Inner { A(bool), B } struct Boxed { inner: Inner, flag: bool } enum Outer { Wrap(Boxed), Unit } fn main(x: Outer) { match x { Outer::Wrap(Boxed { inner: Inner::A(true), flag: true }) => 1, Outer::Wrap(Boxed { inner: Inner::A(true), flag: false }) => 2, Outer::Wrap(Boxed { inner: Inner::A(false), flag: true }) => 3, Outer::Wrap(Boxed { inner: Inner::B, flag: true }) => 5, Outer::Wrap(Boxed { inner: Inner::B, flag: false }) => 6, Outer::Unit => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "nested struct+enum payload non-exhaustive match should fail"
        );
    }

    #[test]
    fn test_enum_payload_recursive_non_finite_requires_catchall_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Nat { Z, S(Nat) } fn main(n: Nat) { match n { Nat::Z => 0, Nat::S(Nat::Z) => 1 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "recursive non-finite payload should require catch-all payload arm"
        );
    }

    #[test]
    fn test_enum_payload_recursive_non_finite_with_catchall_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Nat { Z, S(Nat) } fn main(n: Nat) { match n { Nat::Z => 0, Nat::S(_) => 1 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "recursive non-finite payload should pass with catch-all payload arm"
        );
    }

    #[test]
    fn test_recursive_enum_nested_pattern_decomposition_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Nat { Z, S(Nat) } fn main(n: Nat) { match n { Nat::Z => 0, Nat::S(Nat::Z) => 1, Nat::S(Nat::S(_)) => 2 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "recursive enum nested decomposition should recognize exhaustive split across finite recursive skeleton"
        );
    }

    #[test]
    fn test_recursive_enum_nested_pattern_decomposition_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum Nat { Z, S(Nat) } fn main(n: Nat) { match n { Nat::Z => 0, Nat::S(Nat::Z) => 1 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "recursive enum nested decomposition should fail when recursive remainder branch is missing"
        );
    }

    #[test]
    fn test_recursive_payload_with_finite_bool_decomposition_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum List { Nil, Cons((bool, List)) } fn main(xs: List) { match xs { List::Nil => 0, List::Cons((true, _)) => 1, List::Cons((false, _)) => 2 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "recursive payload should allow finite decomposition over bool with recursive wildcard slot"
        );
    }

    #[test]
    fn test_recursive_payload_with_finite_bool_decomposition_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "enum List { Nil, Cons((bool, List)) } fn main(xs: List) { match xs { List::Nil => 0, List::Cons((true, _)) => 1 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "recursive payload bool decomposition should fail when a finite bool branch is missing"
        );
    }

    #[test]
    fn test_recursive_struct_finite_bool_decomposition_exhaustive_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "struct Node { flag: bool, next: Node } fn main(n: Node) { match n { Node { flag: true, next: _ } => 1, Node { flag: false, next: _ } => 0 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_ok(),
            "recursive struct decomposition over finite bool field should be exhaustive"
        );
    }

    #[test]
    fn test_recursive_struct_finite_bool_decomposition_non_exhaustive_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "struct Node { flag: bool, next: Node } fn main(n: Node) { match n { Node { flag: true, next: _ } => 1 } }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "recursive struct decomposition over finite bool field should fail when a branch is missing"
        );
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
    fn test_unknown_generic_constructor_error_has_position() {
        let parser = Parser::new();
        let ast = parser.parse("fn main() { let x: Foo<i32> = 1; }").unwrap();
        let mut checker = TypeChecker::new();
        let err = checker
            .check(&ast)
            .expect_err("unknown generic constructor should fail type checking");
        assert!(
            err.contains("line") && err.contains("col"),
            "unknown generic constructor error should include line/col, got: {}",
            err
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
    fn test_unknown_trait_type_error_has_position() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn use_it(x: trait Missing) { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let err = checker
            .check(&ast)
            .expect_err("unknown trait type should fail");
        assert!(
            err.contains("line") && err.contains("col"),
            "unknown trait type error should include line/col, got: {}",
            err
        );
    }

    #[test]
    fn test_impl_generic_bound_unknown_param_fails_with_position() {
        let parser = Parser::new();
        let ast = parser
            .parse(
                "trait Show { fn show() {} } impl<T: Show, U: Show> Show for Vec<T> { fn show() {} }",
            )
            .unwrap();
        let mut checker = TypeChecker::new();
        let err = checker.check(&ast).expect_err(
            "impl bounds that reference params not present in impl type head should fail",
        );
        assert!(
            err.contains("line") && err.contains("col"),
            "definition-time impl bound error should include line/col, got: {}",
            err
        );
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
    fn test_where_clause_known_trait_passes() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Show { fn show() {} } fn print_it(x: T) where T: Show { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_ok(), "known where-clause trait should pass");
    }

    #[test]
    fn test_where_clause_unknown_trait_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("fn print_it(x: T) where T: Show { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(result.is_err(), "unknown where-clause trait should fail");
    }

    #[test]
    fn test_where_clause_unknown_type_var_fails() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Show { fn show() {} } fn print_it(x: i32) where T: Show { return; }")
            .unwrap();
        let mut checker = TypeChecker::new();
        let result = checker.check(&ast);
        assert!(
            result.is_err(),
            "where-clause unknown type variable should fail"
        );
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
