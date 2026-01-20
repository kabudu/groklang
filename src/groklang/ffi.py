from typing import Any
from src.groklang.types import Type, PrimitiveType, GenericType

class TypeMarshaler:
    def grok_to_c(self, value: Any, type_: Type) -> Any:
        """Convert GrokLang value to C-compatible with robustness"""
        try:
            if isinstance(type_, PrimitiveType):
                if type_.name == "i32":
                    if not isinstance(value, (int, float)):
                        raise TypeError(f"Expected numeric for i32, got {type(value)}")
                    if not (-2**31 <= value < 2**31):
                        raise ValueError(f"Value {value} out of i32 range")
                    return int(value)
                elif type_.name == "f64":
                    return float(value)
                elif type_.name == "str":
                    if not isinstance(value, str):
                        raise TypeError(f"Expected str, got {type(value)}")
                    return value.encode('utf-8') + b'\x00'
                elif type_.name == "bool":
                    return 1 if value else 0
            elif isinstance(type_, GenericType):
                if type_.name == "Vec":
                    if not hasattr(value, '__len__'):
                        raise TypeError(f"Expected iterable for Vec, got {type(value)}")
                    return (len(value), list(value))  # (len, array)
            return value
        except Exception as e:
            raise RuntimeError(f"FFI marshaling error in grok_to_c: {e}")

    def c_to_grok(self, value: Any, type_: Type) -> Any:
        """Convert C value back to GrokLang with validation"""
        try:
            if isinstance(type_, PrimitiveType):
                if type_.name == "i32":
                    result = int(value)
                    if not (-2**31 <= result < 2**31):
                        raise ValueError(f"C value {value} out of i32 range")
                    return result
                elif type_.name == "f64":
                    return float(value)
                elif type_.name == "str":
                    if isinstance(value, bytes):
                        return value.decode('utf-8').rstrip('\x00')
                    return str(value)
                elif type_.name == "bool":
                    return bool(value)
            elif isinstance(type_, GenericType):
                if type_.name == "Vec" and isinstance(value, tuple):
                    length, array = value
                    return array[:length]  # Slice to prevent overflow
            return value
        except Exception as e:
            raise RuntimeError(f"FFI marshaling error in c_to_grok: {e}")
        return value

class PythonFFI:
    def __init__(self):
        self.modules = {}

    def import_module(self, name: str):
        """Import Python module"""
        import importlib
        self.modules[name] = importlib.import_module(name)
        return self.modules[name]

    def call_function(self, module_name: str, func_name: str, args: list, marshaler: TypeMarshaler) -> Any:
        """Call Python function"""
        if module_name not in self.modules:
            self.import_module(module_name)

        module = self.modules[module_name]
        func = getattr(module, func_name)

        # Marshal args (simplified, assume already marshaled)
        result = func(*args)
        return result

class CFFI:
    def __init__(self):
        self.libs = {}

    def load_library(self, name: str):
        """Load C library (stub)"""
        # In real implementation, use ctypes
        import ctypes
        try:
            self.libs[name] = ctypes.CDLL(name)
        except:
            self.libs[name] = None  # Placeholder
        return self.libs[name]

    def call_function(self, lib_name: str, func_name: str, args: list) -> Any:
        """Call C function (stub)"""
        if lib_name in self.libs and self.libs[lib_name]:
            func = getattr(self.libs[lib_name], func_name)
            return func(*args)
        return None

class BidirectionalFFI:
    def __init__(self):
        self.python_ffi = PythonFFI()
        self.c_ffi = CFFI()
        self.exports = {}  # Grok functions exported to other languages

    def export_function(self, name: str, func):
        """Export Grok function for calling from other languages"""
        self.exports[name] = func

    def call_exported(self, name: str, args: list) -> Any:
        """Call exported Grok function"""
        if name in self.exports:
            return self.exports[name](*args)
        raise ValueError(f"Exported function {name} not found")

    def python_to_grok(self, py_func, grok_func_name: str):
        """Wrap Python function to call Grok (stub)"""
        def wrapper(*args):
            # Simplified: call exported function directly
            return self.call_exported(grok_func_name, list(args))
        return wrapper

class NodeJsFFI:
    def __init__(self):
        self.node_process = None

    def load_module(self, module_path: str):
        """Load Node.js module"""
        # Stub: would start Node.js process or use some bridge
        self.node_process = f"loaded_{module_path}"
        return self.node_process

    def call_function(self, module: str, func_name: str, args: list) -> Any:
        """Call Node.js function (stub)"""
        # Stub: communicate with Node.js process
        if self.node_process:
            return f"nodejs_result_{func_name}({args})"
        return None

class RustFFI:
    def __init__(self):
        self.libs = {}

    def load_library(self, lib_path: str):
        """Load Rust library (compiled to shared lib)"""
        import ctypes
        try:
            self.libs[lib_path] = ctypes.CDLL(lib_path)
        except:
            self.libs[lib_path] = None
        return self.libs[lib_path]

    def call_function(self, lib_path: str, func_name: str, args: list) -> Any:
        """Call Rust function"""
        if lib_path in self.libs and self.libs[lib_path]:
            func = getattr(self.libs[lib_path], func_name)
            return func(*args)
        return None

class GoFFI:
    def __init__(self):
        self.libs = {}

    def load_library(self, lib_path: str):
        """Load Go library"""
        import ctypes
        try:
            self.libs[lib_path] = ctypes.CDLL(lib_path)
        except:
            self.libs[lib_path] = None
        return self.libs[lib_path]

    def call_function(self, lib_path: str, func_name: str, args: list) -> Any:
        """Call Go function"""
        if lib_path in self.libs and self.libs[lib_path]:
            func = getattr(self.libs[lib_path], func_name)
            return func(*args)
        return None