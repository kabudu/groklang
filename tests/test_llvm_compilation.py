from src.groklang.llvm_codegen import LLVMGenerator
import tempfile
import os

def test_llvm_ir_generation():
    """Test basic LLVM IR generation"""
    gen = LLVMGenerator()
    
    # Create a simple function (mock)
    from src.groklang.ir import IRFunction, IRBlock, IRInstruction
    block = IRBlock("entry")
    block.instructions = [IRInstruction("RET", [])]
    func = IRFunction("test", [], [block])
    
    ir_code = gen.generate([func])
    assert "define" in ir_code
    assert "@test" in ir_code
    print("LLVM IR generation test passed!")

def test_compile_to_executable():
    """Test native compilation (requires clang)"""
    gen = LLVMGenerator()
    
    # Create temp file
    with tempfile.NamedTemporaryFile(suffix='.grok', delete=False) as f:
        temp_file = f.name
    
    ir_file = temp_file.replace('.grok', '.ll')
    exe_file = temp_file.replace('.grok', '')
    
    try:
        result = gen.compile_to_executable(temp_file)
        # Check if IR file was created
        assert os.path.exists(ir_file), f"IR file {ir_file} not created"
        
        # If clang succeeded, executable should exist
        if "Successfully compiled" in result:
            assert os.path.exists(exe_file), f"Executable {exe_file} not created"
        
        print("Native compilation test passed!")
    finally:
        # Cleanup
        for f in [temp_file, ir_file, exe_file]:
            if 'f' in locals() and os.path.exists(f):
                os.unlink(f)

if __name__ == "__main__":
    test_llvm_ir_generation()
    test_compile_to_executable()