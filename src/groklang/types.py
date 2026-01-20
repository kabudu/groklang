from dataclasses import dataclass
from typing import Optional, Dict, List

@dataclass
class Type:
    """Base type representation"""
    pass

@dataclass
class PrimitiveType(Type):
    name: str  # "i32", "f64", "bool", "str", etc.

@dataclass
class TypeVariable(Type):
    name: str
    constraints: Optional[List] = None  # Trait bounds

@dataclass
class GenericType(Type):
    name: str
    args: List[Type]  # Type arguments

@dataclass
class FunctionType(Type):
    params: List[Type]
    return_type: Type

@dataclass
class StructType(Type):
    name: str
    fields: Dict[str, Type]  # field_name -> Type

@dataclass
class TraitType(Type):
    name: str
    methods: Dict[str, Optional[FunctionType]]

class TypeEnvironment:
    """Maps identifiers to types"""
    def __init__(self, parent=None):
        self.parent = parent
        self.bindings: Dict[str, Type] = {}

    def bind(self, name: str, type_: Type):
        self.bindings[name] = type_

    def lookup(self, name: str) -> Optional[Type]:
        if name in self.bindings:
            return self.bindings[name]
        elif self.parent:
            return self.parent.lookup(name)
        else:
            return None

    def enter_scope(self):
        return TypeEnvironment(self)

@dataclass
class Constraint:
    left: Type
    right: Type