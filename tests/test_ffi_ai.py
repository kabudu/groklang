from src.groklang.ffi import TypeMarshaler, PythonFFI, CFFI, BidirectionalFFI, NodeJsFFI, RustFFI, GoFFI
from src.groklang.types import PrimitiveType
from src.groklang.decorator_processor import DecoratorProcessor, MockLlmService

def test_type_marshaling():
    marshaler = TypeMarshaler()

    # Test primitive types
    i32_type = PrimitiveType("i32")
    result = marshaler.grok_to_c(42, i32_type)
    assert result == 42

    f64_type = PrimitiveType("f64")
    result = marshaler.grok_to_c(3.14, f64_type)
    assert abs(result - 3.14) < 0.01

    print("Type marshaling test passed!")

def test_python_ffi():
    py_ffi = PythonFFI()

    # Test import (if available)
    try:
        math_module = py_ffi.import_module("math")
        assert hasattr(math_module, "sqrt")
        print("Python FFI import test passed!")
    except ImportError:
        print("Python FFI import test skipped (math not available)")

def test_nodejs_ffi():
    nodejs_ffi = NodeJsFFI()
    
    module = nodejs_ffi.load_module("fs")
    assert module == "loaded_fs"
    
    result = nodejs_ffi.call_function("fs", "readFile", ["test.txt"])
    assert result == "nodejs_result_readFile(['test.txt'])"
    print("Node.js FFI test passed!")

def test_rust_ffi():
    rust_ffi = RustFFI()
    
    # Mock library loading
    lib = rust_ffi.load_library("libmyrust.so")
    # In real scenario, would load actual library
    
    # Since lib is None (stub), call_function returns None
    result = rust_ffi.call_function("libmyrust.so", "my_func", [1, 2])
    assert result is None
    print("Rust FFI test passed!")

def test_go_ffi():
    go_ffi = GoFFI()
    
    lib = go_ffi.load_library("libmygo.so")
    result = go_ffi.call_function("libmygo.so", "my_func", [1, 2])
    assert result is None
    print("Go FFI test passed!")

def test_decorator_processing():
    processor = DecoratorProcessor(MockLlmService())

    # Mock function AST
    class MockFunc:
        def __init__(self):
            self.decorators = ['ai_optimize']
            self.name = 'test_func'

    func = MockFunc()
    result = processor.apply_decorators(func)
    # Should return the same item (mock)
    assert result == func
    print("Decorator processing test passed!")

if __name__ == "__main__":
    test_type_marshaling()
    test_python_ffi()
    test_nodejs_ffi()
    test_rust_ffi()
    test_go_ffi()
    test_decorator_processing()