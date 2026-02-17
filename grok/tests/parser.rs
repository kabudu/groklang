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
            parser.parse("fn main() { let s = \"hello\"; let f = 3.14; let b = b\"bytes\"; }");
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
            r#"fn main() { let h = 0xFF; let b = 0b1010; let o = 0o77; let s = r"C:\tmp\file"; }"#,
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
