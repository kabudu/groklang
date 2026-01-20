from typing import List, Tuple
from .types import Type, TypeEnvironment, FunctionType, StructType, TraitType, PrimitiveType, Constraint, GenericType
from .constraint_collector import ConstraintCollector
from .unifier import Unifier
from .ast_nodes import *

class TypeChecker:
    def __init__(self):
        self.collector = ConstraintCollector()
        self.unifier = Unifier()
        self.type_env = TypeEnvironment()

    def check(self, ast) -> List[Tuple[str, Type]]:
        """Full type check pass"""
        self.check_program(ast)
        substitution = self.unifier.unify(self.collector.constraints)
        return [(var, subst) for var, subst in substitution.items()]

    def check_program(self, program):
        for item in program[1]:  # program is ('Program', items)
            if isinstance(item, FunctionDef):
                self.check_function(item)
            elif isinstance(item, StructDef):
                self.check_struct(item)
            elif isinstance(item, TraitDef):
                self.check_trait(item)

    def check_function(self, func: FunctionDef):
        # Create new scope
        func_env = self.type_env.enter_scope()

        # Bind parameters
        param_types = []
        for param in func.params:
            if param[2]:  # Type annotation provided
                param_type = self.parse_type(param[2])
            else:
                param_type = self.collector.fresh_type_var()

            param_types.append(param_type)
            func_env.bind(param[0], param_type)

        # Infer return type from body
        body_type = self.collector.collect_expr(func.body, func_env)

        # If return type annotation provided, unify
        if func.return_type:
            declared_type = self.parse_type(func.return_type)
            self.collector.constraints.append(Constraint(body_type, declared_type))

        # Store function type
        func_type = FunctionType(param_types, body_type)
        self.type_env.bind(func.name, func_type)

    def check_struct(self, struct: StructDef):
        fields = {}
        for field_name, field_type in struct.fields:
            fields[field_name] = self.parse_type(field_type)

        struct_type = StructType(struct.name, fields)
        self.type_env.bind(struct.name, struct_type)

    def check_trait(self, trait: TraitDef):
        methods = {}
        for method in trait.methods:
            if isinstance(method, FunctionDef):
                # Simplified: just store method name
                methods[method.name] = None

        trait_type = TraitType(trait.name, methods)
        self.type_env.bind(trait.name, trait_type)

    def parse_type(self, type_spec) -> Type:
        """Parse type annotation"""
        if isinstance(type_spec, tuple):
            if type_spec[0] == 'Type':
                return PrimitiveType(type_spec[1])
            elif type_spec[0] == 'GenericType':
                return GenericType(type_spec[1], [self.parse_type(arg) for arg in type_spec[2]])
        return PrimitiveType("unknown")