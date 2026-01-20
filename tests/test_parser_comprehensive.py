from src.groklang import parser
from src.groklang.ast_nodes import *

def test_expressions():
    # Test literals
    code = "fn main() { let x = 42; }"
    ast = parser.parser.parse(code)
    assert ast is not None
    assert len(ast[1]) == 1
    func = ast[1][0]
    assert func.body.statements[0][0] == 'Let'

    code = '"hello"'
    ast = parser.parser.parse(code)
    assert isinstance(ast[1][0], StringLiteral)

    code = "true"
    ast = parser.parser.parse(code)
    assert isinstance(ast[1][0], BoolLiteral)

    # Test binary ops
    code = "a + b"
    ast = parser.parser.parse(code)
    assert isinstance(ast[1][0], BinaryOp)
    assert ast[1][0].op == '+'

    # Test precedence
    code = "a + b * c"
    ast = parser.parser.parse(code)
    assert isinstance(ast[1][0], BinaryOp)
    assert ast[1][0].op == '+'
    assert isinstance(ast[1][0].right, BinaryOp)
    assert ast[1][0].right.op == '*'

def test_function_def():
    code = """
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    """
    ast = parser.parser.parse(code)
    func = ast[1][0]
    assert isinstance(func, FunctionDef)
    assert func.name == 'add'
    assert len(func.params) == 2
    assert func.return_type is not None

def test_struct_def():
    code = """
    struct Point {
        x: i32,
        y: i32,
    }
    """
    ast = parser.parser.parse(code)
    struct = ast[1][0]
    assert isinstance(struct, StructDef)
    assert struct.name == 'Point'
    assert len(struct.fields) == 2

def test_enum_def():
    code = """
    enum Option {
        Some(i32),
        None,
    }
    """
    ast = parser.parser.parse(code)
    enum = ast[1][0]
    assert isinstance(enum, EnumDef)
    assert enum.name == 'Option'
    assert len(enum.variants) == 2

def test_trait_def():
    code = """
    trait Add {
        fn add(self, other: Self) -> Self;
    }
    """
    ast = parser.parser.parse(code)
    trait = ast[1][0]
    assert isinstance(trait, TraitDef)
    assert trait.name == 'Add'

def test_if_expr():
    code = "if x { y } else { z }"
    ast = parser.parser.parse(code)
    if_expr = ast[1][0]
    assert isinstance(if_expr, IfExpr)
    assert if_expr.else_body is not None

def test_match_expr():
    code = """
    match x {
        1 => "one",
        2 => "two",
    }
    """
    ast = parser.parser.parse(code)
    match_expr = ast[1][0]
    assert isinstance(match_expr, MatchExpr)
    assert len(match_expr.arms) == 2

def test_let_statement():
    code = "let x = 42;"
    ast = parser.parser.parse(code)
    stmt = ast[1][0]
    assert stmt[0] == 'Let'
    assert stmt[1] == 'x'

def test_function_call():
    code = "add(1, 2)"
    ast = parser.parser.parse(code)
    call = ast[1][0]
    assert isinstance(call, FunctionCall)
    assert len(call.args) == 2

def test_block():
    code = "{ let x = 1; x + 1 }"
    ast = parser.parser.parse(code)
    block = ast[1][0]
    assert isinstance(block, Block)
    assert len(block.statements) == 2

def run_all_tests():
    test_expressions()
    test_function_def()
    test_struct_def()
    test_enum_def()
    test_trait_def()
    test_if_expr()
    test_match_expr()
    test_let_statement()
    test_function_call()
    test_block()
    print("All parser tests passed!")

if __name__ == "__main__":
    run_all_tests()