from typing import List
from .ir import IRFunction, IRInstruction

class LLVMGenerator:
    def __init__(self):
        self.llvm_code = []

    def generate(self, functions: List[IRFunction]) -> str:
        """Generate LLVM IR from IR functions"""
        self.llvm_code = []
        self.llvm_code.append('; ModuleID = "groklang"')
        self.llvm_code.append('target triple = "x86_64-pc-linux-gnu"')
        self.llvm_code.append('')

        for func in functions:
            self.gen_function(func)

        return '\n'.join(self.llvm_code)

    def gen_function(self, func: IRFunction):
        # Function signature (simplified)
        params = ', '.join(f'i32 %{p[0]}' for p in func.params)
        self.llvm_code.append(f'define i32 @{func.name}({params}) {{')

        # Generate body (simplified)
        for block in func.blocks:
            self.llvm_code.append(f'{block.label}:')
            for i, instr in enumerate(block.instructions):
                llvm_instr = self.gen_instruction(instr, i)
                if llvm_instr:
                    self.llvm_code.append(f'  {llvm_instr}')

        self.llvm_code.append('}')
        self.llvm_code.append('')

    def gen_instruction(self, instr: IRInstruction, index: int):
        opcode = instr.opcode
        args = instr.args

        if opcode == "PUSH_INT":
            return f'%{index} = add i32 0, {args[0]}'
        elif opcode == "ADD":
            return f'%{index} = add i32 %prev1, %prev2'  # Simplified
        # Add more as needed

        return None

    def compile_to_executable(self, filename: str):
        """Compile LLVM IR to native executable"""
        ir_file = filename.replace('.grok', '.ll')
        exe_file = filename.replace('.grok', '')

        # Generate and save IR
        with open(ir_file, 'w') as f:
            f.write(self.generate([]))

        # Compile with clang
        import subprocess
        try:
            # Add runtime library linking if needed
            cmd = ['clang', ir_file, '-o', exe_file]
            subprocess.run(cmd, check=True, capture_output=True)
            return f"Successfully compiled {exe_file}"
        except subprocess.CalledProcessError as e:
            return f"Compilation failed: {e.stderr.decode()}"
        except FileNotFoundError:
            return f"Clang not found. LLVM IR saved to {ir_file}. Install clang for native compilation."