from typing import Optional

class AstNode:
    """Base class for all AST nodes"""
    def __init__(self, line: int, col: int):
        self.line = line
        self.col = col

# Expressions
class IntLiteral(AstNode):
    def __init__(self, value: int, line: int, col: int):
        super().__init__(line, col)
        self.value = value

class FloatLiteral(AstNode):
    def __init__(self, value: float, line: int, col: int):
        super().__init__(line, col)
        self.value = value

class StringLiteral(AstNode):
    def __init__(self, value: str, line: int, col: int):
        super().__init__(line, col)
        self.value = value

class ByteStringLiteral(AstNode):
    def __init__(self, value: bytes, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.value = value

class BoolLiteral(AstNode):
    def __init__(self, value: bool, line: int, col: int):
        super().__init__(line, col)
        self.value = value

class Identifier(AstNode):
    def __init__(self, name: str, line: int, col: int):
        super().__init__(line, col)
        self.name = name

class BinaryOp(AstNode):
    def __init__(self, left, op: str, right, line: int, col: int):
        super().__init__(line, col)
        self.left = left
        self.op = op
        self.right = right

class UnaryOp(AstNode):
    def __init__(self, op: str, operand, line: int, col: int):
        super().__init__(line, col)
        self.op = op
        self.operand = operand

class FunctionDef(AstNode):
    def __init__(self, name: str, params: list, return_type, body, decorators=None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.params = params
        self.return_type = return_type
        self.body = body
        self.decorators = decorators or []

class Block(AstNode):
    def __init__(self, statements: list, line: int, col: int):
        super().__init__(line, col)
        self.statements = statements

class IfExpr(AstNode):
    def __init__(self, condition, then_body, else_body, line: int, col: int):
        super().__init__(line, col)
        self.condition = condition
        self.then_body = then_body
        self.else_body = else_body

class MatchExpr(AstNode):
    def __init__(self, scrutinee, arms: list, line: int, col: int):
        super().__init__(line, col)
        self.scrutinee = scrutinee
        self.arms = arms  # List of (pattern, guard, body)

class FunctionCall(AstNode):
    def __init__(self, func, args: list, line: int, col: int):
        super().__init__(line, col)
        self.func = func
        self.args = args

class StructDef(AstNode):
    def __init__(self, name: str, fields: list, generics: Optional[list] = None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.fields = fields  # List of (name, type)
        self.generics = generics or []

class EnumDef(AstNode):
    def __init__(self, name: str, variants: list, generics: Optional[list] = None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.variants = variants
        self.generics = generics or []

class TraitDef(AstNode):
    def __init__(self, name: str, methods: list, bounds=None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.methods = methods
        self.bounds = bounds or []

class ArrayLiteral(AstNode):
    def __init__(self, elements: list, line: int, col: int):
        super().__init__(line, col)
        self.elements = elements

class TupleLiteral(AstNode):
    def __init__(self, elements: list, line: int, col: int):
        super().__init__(line, col)
        self.elements = elements

class StructLiteral(AstNode):
    def __init__(self, name: str, fields: dict, line: int, col: int):
        super().__init__(line, col)
        self.name = name
        self.fields = fields

class EnumLiteral(AstNode):
    def __init__(self, enum_name: str, variant: str, value, line: int, col: int):
        super().__init__(line, col)
        self.enum_name = enum_name
        self.variant = variant
        self.value = value

class Closure(AstNode):
    def __init__(self, params: list, return_type, body, line: int, col: int):
        super().__init__(line, col)
        self.params = params
        self.return_type = return_type
        self.body = body

class Loop(AstNode):
    def __init__(self, body: Block, line: int, col: int):
        super().__init__(line, col)
        self.body = body

class ForLoop(AstNode):
    def __init__(self, var: str, iterable, body: Block, line: int, col: int):
        super().__init__(line, col)
        self.var = var
        self.iterable = iterable
        self.body = body

class WhileLoop(AstNode):
    def __init__(self, condition, body: Block, line: int, col: int):
        super().__init__(line, col)
        self.condition = condition
        self.body = body

class Return(AstNode):
    def __init__(self, value=None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.value = value

class Break(AstNode):
    def __init__(self, value=None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.value = value

class Continue(AstNode):
    def __init__(self, line: int, col: int):
        super().__init__(line, col)

class Spawn(AstNode):
    def __init__(self, body: Block, line: int, col: int):
        super().__init__(line, col)
        self.body = body