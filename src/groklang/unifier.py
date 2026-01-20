from typing import Dict, List
from .types import Type, TypeVariable, GenericType, PrimitiveType, Constraint

class Unifier:
    def unify(self, constraints: List[Constraint]) -> Dict[str, Type]:
        """Solve type constraints via union-find"""
        substitution: Dict[str, Type] = {}

        for constraint in constraints:
            s_left = self.apply_substitution(constraint.left, substitution)
            s_right = self.apply_substitution(constraint.right, substitution)

            if isinstance(s_left, TypeVariable) and isinstance(s_right, TypeVariable):
                if s_left.name != s_right.name:
                    substitution[s_left.name] = s_right

            elif isinstance(s_left, TypeVariable):
                if not self.occurs_check(s_left.name, s_right):
                    substitution[s_left.name] = s_right

            elif isinstance(s_right, TypeVariable):
                if not self.occurs_check(s_right.name, s_left):
                    substitution[s_right.name] = s_left

            elif isinstance(s_left, GenericType) and isinstance(s_right, GenericType):
                if s_left.name != s_right.name or len(s_left.args) != len(s_right.args):
                    raise TypeError(f"Cannot unify {s_left} with {s_right}")

                for arg_l, arg_r in zip(s_left.args, s_right.args):
                    self.unify([Constraint(arg_l, arg_r)])

            elif isinstance(s_left, PrimitiveType) and isinstance(s_right, PrimitiveType):
                if s_left.name != s_right.name:
                    raise TypeError(f"Type mismatch: {s_left} vs {s_right}")

        return substitution

    def apply_substitution(self, type_: Type, substitution: Dict[str, Type]) -> Type:
        if isinstance(type_, TypeVariable):
            if type_.name in substitution:
                return self.apply_substitution(substitution[type_.name], substitution)
            return type_
        elif isinstance(type_, GenericType):
            return GenericType(
                type_.name,
                [self.apply_substitution(arg, substitution) for arg in type_.args]
            )
        else:
            return type_

    def occurs_check(self, var_name: str, type_: Type) -> bool:
        """Check if type variable occurs in type (prevent infinite types)"""
        if isinstance(type_, TypeVariable):
            return type_.name == var_name
        elif isinstance(type_, GenericType):
            return any(self.occurs_check(var_name, arg) for arg in type_.args)
        return False