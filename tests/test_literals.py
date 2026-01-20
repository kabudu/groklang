import src.groklang.lexer as lexer

def test_raw_string():
    lexer.lexer.input('r"raw\\nstring"')
    tok = lexer.lexer.token()
    assert tok.type == 'RAW_STRING'
    assert tok.value == 'raw\\nstring'
    print("Raw string test passed!")

def test_byte_string():
    lexer.lexer.input('b"byte string"')
    tok = lexer.lexer.token()
    assert tok.type == 'BYTE_STRING'
    assert tok.value == b'byte string'
    print("Byte string test passed!")

def test_string_escapes():
    lexer.lexer.input('"hello world"')
    tok = lexer.lexer.token()
    assert tok.type == 'STRING'
    assert tok.value == 'hello world'
    print("String escapes test passed!")

if __name__ == "__main__":
    test_raw_string()
    test_byte_string()
    test_string_escapes()