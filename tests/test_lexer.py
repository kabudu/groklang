from src.groklang import lexer

def test_lexer():
    code = '''
    // This is a comment
    fn add(a: i32, b: i32) -> i32 {
        let x = 42;  // integer literal
        let y = 3.14;  // float literal
        let z = "hello";  // string literal
        let w = 'c';  // char literal
        let flag = true;  // boolean
        let hex = 0xFF;  // hex integer
        let bin = 0b1010;  // binary
        let oct = 0o777;  // octal
        let big = 123456789i64;  // typed integer
        /* Multi-line comment
           with multiple lines */
        a + b * (c - d)  // operators
    }
    '''

    lexer.lexer.input(code)
    tokens = []
    while True:
        tok = lexer.lexer.token()
        if not tok:
            break
        tokens.append((tok.type, tok.value))
        print(f"{tok.type}: {tok.value}")

    # Basic assertions
    assert any(t[0] == 'FN' for t in tokens)
    assert any(t[0] == 'INT' and t[1] == 42 for t in tokens)
    assert any(t[0] == 'FLOAT' and abs(t[1] - 3.14) < 0.01 for t in tokens)
    assert any(t[0] == 'STRING' and t[1] == 'hello' for t in tokens)
    assert any(t[0] == 'TRUE' for t in tokens)
    assert any(t[0] == 'PLUS' for t in tokens)
    print("Lexer test passed!")

if __name__ == "__main__":
    test_lexer()