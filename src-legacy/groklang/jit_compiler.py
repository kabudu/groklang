import llvmlite.binding as llvm
import ctypes
from .llvm_codegen import LLVMGenerator

class JITCompiler:
    def __init__(self):
        self.engine = None
        self.initialize_jit()

    def initialize_jit(self):
        """Initialize LLVM JIT engine"""
        try:
            llvm.initialize()
            llvm.initialize_native_target()
            llvm.initialize_native_asmprinter()
            llvm.initialize_native_asmparser()
            
            target = llvm.Target.from_default_triple()
            target_machine = target.create_target_machine()
            
            # Create JIT engine
            self.engine = llvm.create_mcjit_compiler(llvm.parse_assembly(""), target_machine)
        except Exception as e:
            print(f"JIT initialization failed: {e}")
            self.engine = None

    def compile_and_run(self, ir_code: str, func_name: str = "main") -> int:
        """JIT compile and execute function, return result"""
        if not self.engine:
            return -1
        
        try:
            # Parse IR
            mod = llvm.parse_assembly(ir_code)
            
            # Add to engine
            self.engine.add_module(mod)
            
            # Get function pointer
            func_ptr = self.engine.get_function_address(func_name)
            
            if func_ptr:
                # Call function (assuming int return)
                func = ctypes.CFUNCTYPE(ctypes.c_int)(func_ptr)
                result = func()
                return result
            else:
                return -1
        except Exception as e:
            print(f"JIT execution failed: {e}")
            return -1