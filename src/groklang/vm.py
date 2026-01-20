from typing import List, Dict, Any
from .ir import IRFunction, IRInstruction

class BytecodeVM:
    def __init__(self):
        self.stack = []
        self.variables: Dict[str, Any] = {}
        self.functions: Dict[str, IRFunction] = {}

    def load_program(self, functions: List[IRFunction]):
        for func in functions:
            self.functions[func.name] = func

    def call_function(self, name: str, args: List) -> Any:
        if name not in self.functions:
            raise ValueError(f"Function {name} not found")

        func = self.functions[name]
        # Bind parameters (simplified)
        for i, param in enumerate(func.params):
            self.variables[param[0]] = args[i]

        # Execute entry block
        return self.execute_block(func.blocks[0])

    def execute_block(self, block) -> Any:
        for instr in block.instructions:
            result = self.execute_instruction(instr)
            if result is not None:
                return result
        return None

    def execute_instruction(self, instr: IRInstruction) -> Any:
        opcode = instr.opcode
        args = instr.args

        if opcode == "PUSH_INT":
            self.stack.append(int(args[0]))
        elif opcode == "PUSH_FLOAT":
            self.stack.append(float(args[0]))
        elif opcode == "PUSH_STR":
            self.stack.append(str(args[0]))
        elif opcode == "PUSH_BOOL":
            self.stack.append(bool(args[0]))
        elif opcode == "LOAD_VAR":
            if args[0] not in self.variables:
                raise ValueError(f"Variable {args[0]} not defined")
            self.stack.append(self.variables[args[0]])
        elif opcode == "STORE_VAR":
            self.variables[args[0]] = self.stack.pop()
        elif opcode == "ADD":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a + b)
        elif opcode == "SUB":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a - b)
        elif opcode == "MUL":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a * b)
        elif opcode == "DIV":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a / b)
        elif opcode == "EQ":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a == b)
        elif opcode == "LT":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a < b)
        elif opcode == "GT":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a > b)
        elif opcode == "LE":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a <= b)
        elif opcode == "GE":
            b = self.stack.pop()
            a = self.stack.pop()
            self.stack.append(a >= b)
        elif opcode == "RET":
            return self.stack.pop() if self.stack else None
        elif opcode == "CALL":
            func_name = args[0]
            num_args = args[1]
            call_args = []
            for _ in range(num_args):
                call_args.append(self.stack.pop())
            call_args.reverse()
            result = self.call_function(func_name, call_args)
            if result is not None:
                self.stack.append(result)
        elif opcode == "JMP_IF_FALSE":
            cond = self.stack.pop()
            if not cond:
                # Jump to block (simplified, assume label)
                pass  # Would need block management
        elif opcode == "JMP":
            # Jump to block
            pass
        else:
            raise ValueError(f"Unknown opcode: {opcode}")

        return None