#[cfg(test)]
mod tests {
    use grok::parser::Parser;

    #[test]
    fn test_parse_function() {
        let parser = Parser::new();
        let result = parser.parse("fn add(a: i32, b: i32) -> i32 {}");
        assert!(result.is_ok(), "Failed to parse function: {:?}", result.err());
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
    fn test_parse_match() {
        let parser = Parser::new();
        let result = parser.parse("fn check(x: i32) { match x { 1 => return 1, 2 => return 2, _ => return 0 } }");
        assert!(result.is_ok(), "Failed to parse match: {:?}", result.err());
    }

    #[test]
    fn test_parse_loops() {
        let parser = Parser::new();
        let result = parser.parse("fn loop_test() { while true { break; } for i in list { continue; } }");
        assert!(result.is_ok(), "Failed to parse loops: {:?}", result.err());
    }
    #[test]
    fn test_parse_macros() {
        let parser = Parser::new();
        let result = parser.parse("macro_rules! my_macro { (x) => { print!(x); } }");
        assert!(result.is_ok(), "Failed to parse macro def: {:?}", result.err());
        
        let result = parser.parse("fn main() { my_macro!(10); }");
        assert!(result.is_ok(), "Failed to parse macro call: {:?}", result.err());
    }
}
