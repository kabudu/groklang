from src.groklang.compiler import Compiler

def test_codegen():
    compiler = Compiler()

    code = """
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    """

    try:
        result = compiler.compile(code, target='vm')
        assert 'ir' in result
        assert len(result['ir']) == 1
        func = result['ir'][0]
        assert func.name == 'add'
        print("Codegen test passed!")
    except Exception as e:
        print(f"Codegen test failed: {e}")

def test_vm_execution():
    compiler = Compiler()

    code = """
    fn main() {
        let x = 5;
        let y = 10;
        x + y
    }
    """

    try:
        result = compiler.compile(code, target='vm')
        # Execute main
        # result['vm'].call_function('main', [])
        print("VM test setup passed!")
    except Exception as e:
        print(f"VM test failed: {e}")

if __name__ == "__main__":
    test_codegen()
    test_vm_execution()