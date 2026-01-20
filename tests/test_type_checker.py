from src.groklang.compiler import Compiler

def test_type_inference():
    compiler = Compiler()

    code = """
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    """

    try:
        result = compiler.compile(code)
        assert result['errors'] == []
        print("Type inference test passed!")
    except Exception as e:
        print(f"Type inference test failed: {e}")

def test_function_call():
    compiler = Compiler()

    code = """
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    fn main() {
        let x = add(1, 2);
    }
    """

    try:
        result = compiler.compile(code)
        assert result['errors'] == []
        print("Function call test passed!")
    except Exception as e:
        print(f"Function call test failed: {e}")

if __name__ == "__main__":
    test_type_inference()
    test_function_call()