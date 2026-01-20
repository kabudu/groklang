from src.groklang import parser

def test_parser():
    code = '''
    fn main() {
        let x = 42;
    }
    '''

    ast = parser.parser.parse(code)
    print(ast)
    assert ast is not None
    assert parser.parser.errors == []

if __name__ == "__main__":
    test_parser()