from typing import List
from .ir import IRInstruction, IRBlock, IRFunction
from .ast_nodes import *

class CodeGenerator:
    def __init__(self):
        self.temp_counter = 0
        self.blocks = []  # Current function blocks

    def generate(self, typed_ast) -> List[IRFunction]:
        """Generate IR from typed AST"""
        functions = []
        for item in typed_ast[1]:  # program items
            if isinstance(item, FunctionDef):
                ir_func = self.gen_function(item)
                functions.append(ir_func)
        return functions

    def gen_function(self, func: FunctionDef) -> IRFunction:
        self.blocks = [IRBlock("entry")]
        self.gen_expr(func.body, self.blocks[0])
        return IRFunction(func.name, func.params, self.blocks)

    def gen_expr(self, expr, block: IRBlock):
        if isinstance(expr, IntLiteral):
            block.instructions.append(IRInstruction("PUSH_INT", [expr.value]))
        elif isinstance(expr, FloatLiteral):
            block.instructions.append(IRInstruction("PUSH_FLOAT", [expr.value]))
        elif isinstance(expr, StringLiteral):
            block.instructions.append(IRInstruction("PUSH_STR", [expr.value]))
        elif isinstance(expr, BoolLiteral):
            block.instructions.append(IRInstruction("PUSH_BOOL", [expr.value]))
        elif isinstance(expr, Identifier):
            block.instructions.append(IRInstruction("LOAD_VAR", [expr.name]))
        elif isinstance(expr, BinaryOp):
            self.gen_expr(expr.left, block)
            self.gen_expr(expr.right, block)
            if expr.op == '+':
                block.instructions.append(IRInstruction("ADD", []))
            elif expr.op == '-':
                block.instructions.append(IRInstruction("SUB", []))
            elif expr.op == '*':
                block.instructions.append(IRInstruction("MUL", []))
            elif expr.op == '/':
                block.instructions.append(IRInstruction("DIV", []))
            elif expr.op == '==':
                block.instructions.append(IRInstruction("EQ", []))
            elif expr.op == '<':
                block.instructions.append(IRInstruction("LT", []))
        elif isinstance(expr, FunctionCall):
            for arg in expr.args:
                self.gen_expr(arg, block)
            block.instructions.append(IRInstruction("CALL", [expr.func.name, len(expr.args)]))
        elif isinstance(expr, IfExpr):
            # Generate if-then-else
            then_block = IRBlock(f"then_{self.temp_counter}")
            else_block = IRBlock(f"else_{self.temp_counter}")
            end_block = IRBlock(f"end_{self.temp_counter}")
            self.temp_counter += 1

            # Condition
            self.gen_expr(expr.condition, block)
            block.instructions.append(IRInstruction("JMP_IF_FALSE", [else_block.label]))

            # Then branch
            self.blocks.append(then_block)
            self.gen_expr(expr.then_body, then_block)
            then_block.instructions.append(IRInstruction("JMP", [end_block.label]))

            # Else branch
            self.blocks.append(else_block)
            if expr.else_body:
                self.gen_expr(expr.else_body, else_block)
            else_block.instructions.append(IRInstruction("JMP", [end_block.label]))

            # End block
            self.blocks.append(end_block)
        elif isinstance(expr, Block):
            for stmt in expr.statements:
                if isinstance(stmt, tuple) and stmt[0] == 'Let':
                    # Let statement
                    var_name, var_expr = stmt[1], stmt[3]
                    self.gen_expr(var_expr, block)
                    block.instructions.append(IRInstruction("STORE_VAR", [var_name]))
                else:
                    # Expression statement
                    self.gen_expr(stmt, block)

    def fresh_temp(self) -> str:
        temp = f"t{self.temp_counter}"
        self.temp_counter += 1
        return temp