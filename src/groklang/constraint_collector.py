from typing import List
from .types import Type, TypeVariable, PrimitiveType, FunctionType, Constraint, TypeEnvironment
from .ast_nodes import *

class ConstraintCollector:
    def __init__(self):
        self.constraints: List[Constraint] = []
        self.type_var_counter = 0

    def fresh_type_var(self) -> TypeVariable:
        """Generate fresh type variable"""
        name = f"T{self.type_var_counter}"
        self.type_var_counter += 1
        return TypeVariable(name)

    def collect_expr(self, expr, env: TypeEnvironment) -> Type:
        if isinstance(expr, IntLiteral):
            return PrimitiveType("i32")

        elif isinstance(expr, FloatLiteral):
            return PrimitiveType("f64")

        elif isinstance(expr, StringLiteral):
            return PrimitiveType("str")

        elif isinstance(expr, BoolLiteral):
            return PrimitiveType("bool")

        elif isinstance(expr, Identifier):
            type_ = env.lookup(expr.name)
            if type_ is None:
                raise TypeError(f"Unknown variable: {expr.name}")
            return type_

        elif isinstance(expr, BinaryOp):
            left_type = self.collect_expr(expr.left, env)
            right_type = self.collect_expr(expr.right, env)

            # Constraint: left_type == right_type for arithmetic ops
            if expr.op in ['+', '-', '*', '/', '%']:
                self.constraints.append(Constraint(left_type, right_type))
                return left_type  # Result is same type

            # Boolean ops
            elif expr.op in ['&&', '||']:
                return PrimitiveType("bool")

            # Comparison
            elif expr.op in ['<', '>', '<=', '>=', '==', '!=']:
                return PrimitiveType("bool")

        elif isinstance(expr, FunctionCall):
            func_type = self.collect_expr(expr.func, env)
            if not isinstance(func_type, FunctionType):
                raise TypeError(f"Not a function: {expr.func}")

            for arg, param_type in zip(expr.args, func_type.params):
                arg_type = self.collect_expr(arg, env)
                self.constraints.append(Constraint(arg_type, param_type))

            return func_type.return_type

        elif isinstance(expr, IfExpr):
            cond_type = self.collect_expr(expr.condition, env)
            self.constraints.append(Constraint(cond_type, PrimitiveType("bool")))

            then_type = self.collect_expr(expr.then_body, env)
            if expr.else_body:
                else_type = self.collect_expr(expr.else_body, env)
                self.constraints.append(Constraint(then_type, else_type))
                return then_type
            else:
                return PrimitiveType("unit")  # void

        elif isinstance(expr, Block):
            if expr.statements:
                for stmt in expr.statements:
                    if isinstance(stmt, tuple) and stmt[0] == 'Let':
                        # Let statement
                        var_name, var_expr = stmt[1], stmt[3]
                        var_type = self.collect_expr(var_expr, env)
                        env.bind(var_name, var_type)
                    else:
                        # Expression statement
                        self.collect_expr(stmt, env)
                # Blocks return unit (no last expression type)
                return PrimitiveType("unit")
            else:
                return PrimitiveType("unit")

        else:
            return self.fresh_type_var()