# Phase 2: Type Checker and Decorator Processing

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Target**: Full type inference and AI decorator expansion  
**Duration**: 4-5 weeks

---

## 1. Overview

Implement Hindley-Milner type inference and compile-time AI decorator processing.

```
Typed AST ← Type Checker ← AST (from Phase 1)
Decorated AST ← Decorator Processor ← Typed AST
```

---

## 2. Type Checker Implementation

### 2.1 Type Representation

```python
from dataclasses import dataclass
from enum import Enum

@dataclass
class Type:
    """Base type representation"""
    pass

@dataclass
class PrimitiveType(Type):
    name: str  # "i32", "f64", "bool", etc.

@dataclass
class TypeVariable(Type):
    name: str
    constraints: list = None  # Trait bounds

@dataclass
class GenericType(Type):
    name: str
    args: list  # Type arguments

@dataclass
class FunctionType(Type):
    params: list
    return_type: Type

@dataclass
class StructType(Type):
    name: str
    fields: dict  # field_name -> Type

@dataclass
class TraitType(Type):
    name: str
    methods: dict

class TypeEnvironment:
    """Maps identifiers to types"""
    def __init__(self, parent=None):
        self.parent = parent
        self.bindings = {}

    def bind(self, name: str, type_: Type):
        self.bindings[name] = type_

    def lookup(self, name: str):
        if name in self.bindings:
            return self.bindings[name]
        elif self.parent:
            return self.parent.lookup(name)
        else:
            return None

    def enter_scope(self):
        return TypeEnvironment(self)
```

### 2.2 Constraint Generation

```python
class Constraint:
    def __init__(self, left: Type, right: Type):
        self.left = left
        self.right = right

class ConstraintCollector:
    def __init__(self):
        self.constraints = []
        self.type_var_counter = 0

    def fresh_type_var(self):
        """Generate fresh type variable"""
        name = f"T{self.type_var_counter}"
        self.type_var_counter += 1
        return TypeVariable(name)

    def collect_expr(self, expr, env: TypeEnvironment):
        if isinstance(expr, IntLiteral):
            return PrimitiveType("i32")

        elif isinstance(expr, FloatLiteral):
            return PrimitiveType("f64")

        elif isinstance(expr, StringLiteral):
            return PrimitiveType("str")

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

        else:
            return self.fresh_type_var()
```

### 2.3 Unification Algorithm

```python
class Unifier:
    def unify(self, constraints: list) -> dict:
        """Solve type constraints via union-find"""
        substitution = {}

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
                    self.constraints.append(Constraint(arg_l, arg_r))

            elif isinstance(s_left, PrimitiveType) and isinstance(s_right, PrimitiveType):
                if s_left.name != s_right.name:
                    raise TypeError(f"Type mismatch: {s_left} vs {s_right}")

        return substitution

    def apply_substitution(self, type_: Type, substitution: dict):
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

    def occurs_check(self, var_name: str, type_: Type):
        """Check if type variable occurs in type (prevent infinite types)"""
        if isinstance(type_, TypeVariable):
            return type_.name == var_name
        elif isinstance(type_, GenericType):
            return any(self.occurs_check(var_name, arg) for arg in type_.args)
        return False
```

### 2.4 Type Checking Pass

```python
class TypeChecker:
    def __init__(self):
        self.collector = ConstraintCollector()
        self.unifier = Unifier()
        self.type_env = TypeEnvironment()

    def check(self, ast):
        """Full type check pass"""
        self.check_program(ast)
        return self.collector.constraints

    def check_program(self, program):
        for item in program.items:
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
        body_type = self.collect_expr(func.body, func_env)

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
            # Type each method
            methods[method.name] = None  # Simplified

        trait_type = TraitType(trait.name, methods)
        self.type_env.bind(trait.name, trait_type)

    def collect_expr(self, expr, env):
        return self.collector.collect_expr(expr, env)

    def parse_type(self, type_spec):
        """Parse type annotation"""
        if isinstance(type_spec, tuple):
            if type_spec[0] == 'Type':
                return PrimitiveType(type_spec[1])
            elif type_spec[0] == 'GenericType':
                return GenericType(type_spec[1], [self.parse_type(arg) for arg in type_spec[2]])
        return PrimitiveType("unknown")
```

---

## 3. Decorator Processing

### 3.1 Decorator Extraction

```python
class DecoratorProcessor:
    def __init__(self, llm_service):
        self.llm_service = llm_service  # AI backend

    def process_decorators(self, ast):
        """Expand compile-time decorators"""
        new_ast = []

        for item in ast.items:
            if hasattr(item, 'decorators') and item.decorators:
                transformed = self.apply_decorators(item)
                new_ast.append(transformed)
            else:
                new_ast.append(item)

        return new_ast

    def apply_decorators(self, item):
        """Apply each decorator in sequence"""
        current = item

        for decorator in item.decorators:
            if decorator.name == 'ai_optimize':
                current = self.optimize_decorator(current, decorator)
            elif decorator.name == 'ai_test':
                current = self.test_decorator(current, decorator)
            elif decorator.name == 'ai_translate':
                current = self.translate_decorator(current, decorator)

        return current

    def optimize_decorator(self, item, decorator):
        """Call AI to optimize function"""
        request = {
            'operation': 'optimize',
            'input': ast_to_code(item),
            'parameters': {
                'level': decorator.params.get('level', 'intermediate'),
                'target': decorator.params.get('target', 'speed'),
            }
        }

        response = self.llm_service.call(request)
        if response.success:
            optimized_code = response.output
            optimized_ast = parse(optimized_code)

            # Validate: semantic equivalence check
            if self.validate_optimization(item, optimized_ast):
                return optimized_ast
            else:
                # Fallback: use original
                return item
        else:
            return item

    def test_decorator(self, item, decorator):
        """Generate tests via AI"""
        request = {
            'operation': 'test',
            'input': ast_to_code(item),
            'parameters': {
                'iterations': decorator.params.get('iterations', '100'),
            }
        }

        response = self.llm_service.call(request)
        if response.success:
            # Create test functions from response
            tests = parse_tests(response.output)
            return (item, tests)  # Return function + generated tests
        return (item, [])

    def translate_decorator(self, item, decorator):
        """Translate to target language (e.g., Python)"""
        target = decorator.params.get('target_lang', 'py')

        request = {
            'operation': 'translate',
            'input': ast_to_code(item),
            'parameters': {
                'target_lang': target,
            }
        }

        response = self.llm_service.call(request)
        if response.success:
            return (item, response.output)  # Function + translation
        return (item, None)

    def validate_optimization(self, original, optimized):
        """Validate semantic equivalence"""
        # Simplified: compare AST structure
        original_code = ast_to_code(original)
        optimized_code = ast_to_code(optimized)

        # Run both on test cases
        test_inputs = self.generate_test_inputs(original)

        for input_val in test_inputs:
            orig_result = self.execute(original_code, input_val)
            opt_result = self.execute(optimized_code, input_val)

            if orig_result != opt_result:
                return False  # Semantics changed

        return True
```

### 3.2 LLM Service Abstraction

```python
class LlmService:
    def call(self, request: dict) -> dict:
        """Call LLM (abstract)"""
        raise NotImplementedError

class LocalLlmService(LlmService):
    def __init__(self, model_url: str):
        self.model_url = model_url

    def call(self, request: dict) -> dict:
        import requests

        payload = {
            'prompt': self.format_prompt(request),
            'temperature': 0.7,
        }

        try:
            response = requests.post(f"{self.model_url}/api/generate", json=payload, timeout=5)
            return {'success': True, 'output': response.json()['response']}
        except:
            return {'success': False, 'error': 'Service unavailable'}

    def format_prompt(self, request: dict) -> str:
        op = request['operation']
        code = request['input']

        if op == 'optimize':
            return f"Optimize this function for speed:\n\n{code}\n\nProvide only the optimized code."
        elif op == 'test':
            return f"Generate comprehensive test cases for:\n\n{code}\n\nProvide tests in GrokLang format."
        else:
            return code

class OpenAiService(LlmService):
    def __init__(self, api_key: str, model: str = "gpt-4"):
        self.api_key = api_key
        self.model = model

    def call(self, request: dict) -> dict:
        import openai
        openai.api_key = self.api_key

        try:
            response = openai.ChatCompletion.create(
                model=self.model,
                messages=[{"role": "user", "content": self.format_prompt(request)}],
                temperature=0.7,
                timeout=5,
            )
            return {'success': True, 'output': response.choices[0].message.content}
        except Exception as e:
            return {'success': False, 'error': str(e)}
```

---

## 4. Deliverables

- [x] Type inference engine (Hindley-Milner)
- [x] Constraint generation and unification
- [x] Type environment and scope tracking
- [x] Decorator processor framework
- [x] AI service abstraction
- [x] Optimization validation
- [ ] **Validation (next document)**

---

## 5. Next Steps

→ Proceed to Phase 3: Code Generation

---

## Note on AI Integration

**Configuration (grok.toml)**:

```toml
[ai]
backend = "offline"  # Start offline, add LLM later
compile_time = true
timeout = 5
```

This allows Phase 2 to work without AI initially, adding it later.
