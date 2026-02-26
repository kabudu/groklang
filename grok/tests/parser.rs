#[cfg(test)]
mod tests {
    use grok::ast::AstNode;
    use grok::parser::Parser;

    #[test]
    fn test_parse_function() {
        let parser = Parser::new();
        let result = parser.parse("fn add(a: i32, b: i32) -> i32 {}");
        assert!(
            result.is_ok(),
            "Failed to parse function: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_struct() {
        let parser = Parser::new();
        let result = parser.parse("struct Point { x: i32, y: i32 }");
        assert!(result.is_ok(), "Failed to parse struct: {:?}", result.err());
    }

    #[test]
    fn test_parse_enum() {
        let parser = Parser::new();
        let result = parser.parse("enum Color { Red, Green, Blue, RGB(i32) }");
        assert!(result.is_ok(), "Failed to parse enum: {:?}", result.err());
    }

    #[test]
    fn test_parse_actor() {
        let parser = Parser::new();
        let result = parser.parse("actor MyActor { let x = 1; }");
        assert!(result.is_ok(), "Failed to parse actor: {:?}", result.err());
    }

    #[test]
    fn test_parse_trait() {
        let parser = Parser::new();
        let result = parser.parse("trait Drawable { fn draw() {} }");
        assert!(result.is_ok(), "Failed to parse trait: {:?}", result.err());
    }

    #[test]
    fn test_parse_trait_with_bounds() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Drawable: Renderable + Sized { fn draw() {} }")
            .expect("trait with bounds should parse");

        match ast {
            AstNode::Program(nodes) => match &nodes[0] {
                AstNode::TraitDef { bounds, .. } => {
                    assert_eq!(bounds, &vec!["Renderable".to_string(), "Sized".to_string()])
                }
                _ => panic!("expected trait definition"),
            },
            _ => panic!("expected program"),
        }
    }

    #[test]
    fn test_parse_function_where_clause() {
        let parser = Parser::new();
        let ast = parser
            .parse("trait Show { fn show() {} } fn print_it(x: T) where T: Show { return; }")
            .expect("function with where-clause should parse");

        match ast {
            AstNode::Program(nodes) => match &nodes[1] {
                AstNode::FunctionDef { decorators, .. } => {
                    assert!(
                        decorators.iter().any(|d| d == "__where:T:Show"),
                        "where metadata not captured in decorators: {:?}",
                        decorators
                    );
                }
                _ => panic!("expected function definition"),
            },
            _ => panic!("expected program"),
        }
    }

    #[test]
    fn test_parse_use_statement() {
        let parser = Parser::new();
        let result = parser.parse("use std::io::print;");
        assert!(result.is_ok(), "Failed to parse use statement: {:?}", result.err());
    }

    #[test]
    fn test_parse_use_group_and_glob() {
        let parser = Parser::new();
        let group = parser.parse("use std::io::{print as p, read};");
        assert!(group.is_ok(), "Failed to parse grouped use: {:?}", group.err());

        let glob = parser.parse("use std::io::*;");
        assert!(glob.is_ok(), "Failed to parse glob use: {:?}", glob.err());
    }

    #[test]
    fn test_parse_module_definition() {
        let parser = Parser::new();
        let result = parser.parse("mod math { fn add(a: i32, b: i32) -> i32 { a + b } }");
        assert!(
            result.is_ok(),
            "Failed to parse module definition: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_module_declaration() {
        let parser = Parser::new();
        let result = parser.parse("mod math;");
        assert!(
            result.is_ok(),
            "Failed to parse module declaration: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_match() {
        let parser = Parser::new();
        let result = parser
            .parse("fn check(x: i32) { match x { 1 => return 1, 2 => return 2, _ => return 0 } }");
        assert!(result.is_ok(), "Failed to parse match: {:?}", result.err());
    }

    #[test]
    fn test_parse_loops() {
        let parser = Parser::new();
        let result =
            parser.parse("fn loop_test() { while true { break; } for i in list { continue; } }");
        assert!(result.is_ok(), "Failed to parse loops: {:?}", result.err());
    }
    #[test]
    fn test_parse_macros() {
        let parser = Parser::new();
        let result = parser.parse("macro_rules! my_macro { (x) => { print!(x); } }");
        assert!(
            result.is_ok(),
            "Failed to parse macro def: {:?}",
            result.err()
        );

        let result = parser.parse("fn main() { my_macro!(10); }");
        assert!(
            result.is_ok(),
            "Failed to parse macro call: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let parser = Parser::new();
        let result = parser.parse("fn (a: i32) {}"); // Missing name
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_with_digits() {
        let parser = Parser::new();
        let result = parser.parse("fn f1(a1: i32) -> i32 { let x2 = a1; x2 }");
        assert!(
            result.is_ok(),
            "Failed to parse identifiers with digits: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_expression_precedence() {
        let parser = Parser::new();
        let result = parser.parse("fn main() { let x = 1 + 2 * 3; let y = x >= 3 && x != 7; }");
        assert!(
            result.is_ok(),
            "Failed to parse precedence expression: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_string_float_and_byte_literals() {
        let parser = Parser::new();
        let result =
            parser.parse("fn main() { let s = \"hello\"; let f = 3.14; let c = 'a'; let b = b\"bytes\"; }");
        assert!(
            result.is_ok(),
            "Failed to parse literals: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_reference_and_generic_types() {
        let parser = Parser::new();
        let result = parser.parse("fn id(x: &mut Vec<i32>) -> () { return; }");
        assert!(
            result.is_ok(),
            "Failed to parse generic/reference/unit types: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_bitwise_shift_and_assignment_ops() {
        let parser = Parser::new();
        let result = parser.parse(
            "fn main() { let x = 1 << 2 | 3 ^ 4 & 5; let y = x >> 1; let z = 1; z += y; z &= 7; }",
        );
        assert!(
            result.is_ok(),
            "Failed to parse bitwise/shift/assignment ops: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_index_access() {
        let parser = Parser::new();
        let result = parser.parse("fn main(v: Vec<i32>) { let x = v[0]; }");
        assert!(
            result.is_ok(),
            "Failed to parse index access expression: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_try_operator() {
        let parser = Parser::new();
        let result = parser.parse("fn main(v: Result<i32, i32>) -> i32 { v? }");
        assert!(
            result.is_ok(),
            "Failed to parse try operator expression: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_array_literal() {
        let parser = Parser::new();
        let result = parser.parse("fn main() { let xs = [1, 2, 3]; }");
        assert!(
            result.is_ok(),
            "Failed to parse array literal expression: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_closure_expression() {
        let parser = Parser::new();
        let result = parser.parse("fn main() { let f = |x: i32| x + 1; let g = move |y: i32| -> i32 { y }; }");
        assert!(
            result.is_ok(),
            "Failed to parse closure expressions: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_tuple_literal() {
        let parser = Parser::new();
        let result = parser.parse("fn main() { let p = (1, true); }");
        assert!(
            result.is_ok(),
            "Failed to parse tuple literal expression: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_expanded_patterns() {
        let parser = Parser::new();
        let result = parser.parse(
            "fn main(v: i32) { match v { Point { x: x1, y } => x1, Color::RGB(n) => n, (a, b) => a, _ => 0 } }",
        );
        assert!(
            result.is_ok(),
            "Failed to parse expanded patterns: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_invalid_assignment_rhs() {
        let parser = Parser::new();
        let result = parser.parse("fn main() { let x = 1; x += ; }");
        assert!(result.is_err(), "Invalid assignment should fail parsing");
    }

    #[test]
    fn test_parse_invalid_struct_pattern_field() {
        let parser = Parser::new();
        let result = parser.parse("fn main(v: i32) { match v { Point { x: } => 1, _ => 0 } }");
        assert!(
            result.is_err(),
            "Invalid struct pattern should fail parsing"
        );
    }

    #[test]
    fn test_parse_or_pattern() {
        let parser = Parser::new();
        let result = parser.parse("fn main(v: i32) { match v { 1 | 2 | 3 => 1, _ => 0 } }");
        assert!(
            result.is_ok(),
            "Failed to parse or-pattern: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_prefixed_int_and_raw_string_literals() {
        let parser = Parser::new();
        let result = parser.parse(
            r#"fn main() { let h = 0xFFi64; let b = 0b1010i32; let o = 0o77i32; let n = 42i64; let f = 3.14f32; let s = r"C:\tmp\file"; }"#,
        );
        assert!(
            result.is_ok(),
            "Failed to parse prefixed int/raw string literals: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_trait_type_annotation() {
        let parser = Parser::new();
        let result = parser.parse("trait Printable { fn p() {} } fn show(x: trait Printable) {}");
        assert!(
            result.is_ok(),
            "Failed to parse trait type annotation: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_impl_blocks() {
        let parser = Parser::new();
        let result = parser.parse(
            "trait Drawable { fn draw() {} } struct Point { x: i32 } impl Drawable for Point { fn draw() {} } impl Point { fn norm() {} }",
        );
        assert!(
            result.is_ok(),
            "Failed to parse impl blocks: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_generic_impl_with_bounds() {
        let parser = Parser::new();
        let result = parser.parse(
            "trait Show { fn show() {} } impl<T: Show> Show for Vec<T> { fn show() {} }",
        );
        assert!(
            result.is_ok(),
            "Failed to parse generic impl with bounds: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_detailed_error_position() {
        let parser = Parser::new();
        let err = parser
            .parse_detailed("fn main() {\n  let x = ;\n}")
            .expect_err("expected parse error");
        assert!(err.line >= 1, "line should be populated");
        assert!(err.col >= 1, "column should be populated");
        assert!(err.offset > 0, "offset should be populated");
    }
}
