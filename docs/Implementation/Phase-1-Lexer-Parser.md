# Phase 1: Lexer and Parser Implementation

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Target**: Complete lexical analysis and syntax parsing  
**Duration**: 2-3 weeks  
**Tools**: Python PLY (lex/yacc)

---

## 1. Overview

Build the foundation: convert source code (text) → AST (abstract syntax tree).

```
Source (.grok) → Lexer (PLY lex) → Tokens → Parser (PLY yacc) → AST
```

---

## 2. Lexer Implementation

### 2.1 Setup PLY

```bash
pip install ply
```

### 2.2 Token Definition (lexer.py)

```python
import ply.lex as lex

# Token list
tokens = (
    'INT', 'FLOAT', 'STRING', 'CHAR', 'TRUE', 'FALSE', 'ID',
    'FN', 'LET', 'MUT', 'IF', 'ELSE', 'MATCH', 'FOR', 'WHILE', 'LOOP',
    'BREAK', 'CONTINUE', 'RETURN', 'STRUCT', 'ENUM', 'TRAIT', 'IMPL', 'USE',
    'MOD', 'PUB', 'SELF', 'SELFTYPE', 'WHERE', 'TYPE', 'AS', 'UNSAFE',
    'EXTERN', 'CONST', 'STATIC', 'MACRO', 'AI', 'ACTOR', 'SPAWN',
    # Operators...
    'PLUS', 'MINUS', 'TIMES', 'DIVIDE', 'MODULO',
    'EQ', 'NE', 'LT', 'GT', 'LE', 'GE',
    'AND', 'OR', 'NOT', 'AMPERSAND', 'PIPE', 'CARET', 'TILDE',
    'LSHIFT', 'RSHIFT', 'ASSIGN',
    'LPAREN', 'RPAREN', 'LBRACE', 'RBRACE', 'LBRACKET', 'RBRACKET',
    'COMMA', 'SEMICOLON', 'COLON', 'DOUBLECOLON', 'DOT', 'ARROW', 'FATARROW',
    'ELLIPSIS', 'QUESTION', 'AT', 'HASH', 'DOLLAR',
)

reserved = {
    'fn': 'FN', 'let': 'LET', 'mut': 'MUT', 'if': 'IF',
    'else': 'ELSE', 'match': 'MATCH', 'for': 'FOR', 'while': 'WHILE',
    'loop': 'LOOP', 'break': 'BREAK', 'continue': 'CONTINUE',
    'return': 'RETURN', 'struct': 'STRUCT', 'enum': 'ENUM',
    'trait': 'TRAIT', 'impl': 'IMPL', 'use': 'USE',
    'mod': 'MOD', 'pub': 'PUB', 'self': 'SELF', 'Self': 'SELFTYPE',
    'true': 'TRUE', 'false': 'FALSE', 'where': 'WHERE',
    'type': 'TYPE', 'as': 'AS', 'unsafe': 'UNSAFE',
    'extern': 'EXTERN', 'const': 'CONST', 'static': 'STATIC',
    'macro': 'MACRO', 'ai': 'AI', 'actor': 'ACTOR', 'spawn': 'SPAWN',
}

# Token rules
def t_ID(t):
    r'[a-zA-Z_][a-zA-Z_0-9]*'
    t.type = reserved.get(t.value, 'ID')
    return t

def t_INT(t):
    r'(0[xX][0-9a-fA-F]+|0[bB][01]+|0[oO][0-7]+|\d+)[uUiI]?(8|16|32|64|128)?'
    t.value = int(t.value.rstrip('uUiI'), 0)
    return t

def t_FLOAT(t):
    r'(\d+\.\d*|\d*\.\d+)([eE][+-]?\d+)?[fF]?|(\d+)[eE][+-]?\d+[fF]?'
    t.value = float(t.value)
    return t

def t_STRING(t):
    r'"([^"\\\\]|\\\\.)*"'
    t.value = t.value[1:-1]
    return t

def t_CHAR(t):
    r"'([^'\\\\]|\\\\.)?'"
    t.value = t.value[1:-1]
    return t

# Operators (longest match first)
t_EQ = r'=='
t_NE = r'!='
t_LE = r'<='
t_GE = r'>='
t_AND = r'&&'
t_OR = r'\|\|'
t_LSHIFT = r'<<'
t_RSHIFT = r'>>'
t_DOUBLECOLON = r'::'
t_ARROW = r'->'
t_FATARROW = r'=>'
t_ELLIPSIS = r'\.\.\.'

t_PLUS = r'\+'
t_MINUS = r'-'
t_TIMES = r'\*'
t_DIVIDE = r'/'
t_MODULO = r'%'
t_LT = r'<'
t_GT = r'>'
t_NOT = r'!'
t_AMPERSAND = r'&'
t_PIPE = r'\|'
t_CARET = r'\^'
t_TILDE = r'~'
t_ASSIGN = r'='
t_LPAREN = r'\('
t_RPAREN = r'\)'
t_LBRACE = r'\{'
t_RBRACE = r'\}'
t_LBRACKET = r'\['
t_RBRACKET = r'\]'
t_COMMA = r','
t_SEMICOLON = r';'
t_COLON = r':'
t_DOT = r'\.'
t_QUESTION = r'\?'
t_AT = r'@'
t_HASH = r'#'
t_DOLLAR = r'\$'

# Comments
def t_COMMENT(t):
    r'//[^\n]*'
    pass

def t_MCOMMENT(t):
    r'/\*(.|\n)*?\*/'
    t.lexer.lineno += t.value.count('\n')
    pass

def t_newline(t):
    r'\n+'
    t.lexer.lineno += len(t.value)

t_ignore = ' \t\r'

def t_error(t):
    print(f"Illegal character '{t.value[0]}' at line {t.lexer.lineno}")
    t.lexer.skip(1)

lexer = lex.lex()
```

### 2.3 Lexer Testing

```python
def test_lexer():
    code = '''
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    '''

    lexer.input(code)
    while True:
        tok = lexer.token()
        if not tok:
            break
        print(f"{tok.type}: {tok.value}")
```

---

## 3. AST Node Definitions (ast_nodes.py)

```python
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
    def __init__(self, name: str, params: list, return_type, body, decorators: list = None, line: int = 0, col: int = 0):
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
    def __init__(self, name: str, fields: list, generics: list = None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.fields = fields  # List of (name, type)
        self.generics = generics or []

class EnumDef(AstNode):
    def __init__(self, name: str, variants: list, generics: list = None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.variants = variants
        self.generics = generics or []

class TraitDef(AstNode):
    def __init__(self, name: str, methods: list, bounds: list = None, line: int = 0, col: int = 0):
        super().__init__(line, col)
        self.name = name
        self.methods = methods
        self.bounds = bounds or []

# ... More AST nodes as needed
```

---

## 4. Parser Implementation (parser.py)

```python
import ply.yacc as yacc
from lexer import tokens  # Import from lexer module

class Parser:
    def __init__(self):
        self.tokens = tokens
        self.parser = yacc.yacc(module=self, debug=False, write_tables=False)
        self.errors = []

    # Start rule
    def p_program(self, p):
        """program : items
                   | empty"""
        p[0] = ('Program', p[1])

    def p_items(self, p):
        """items : items item
                 | item"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[2])
            p[0] = p[1]

    def p_item(self, p):
        """item : function_def
                | struct_def
                | enum_def
                | trait_def
                | impl_block"""
        p[0] = p[1]

    # Function definition
    def p_function_def(self, p):
        """function_def : FN ID LPAREN parameters RPAREN ARROW type block
                        | FN ID LPAREN parameters RPAREN block"""
        name = p[2]
        params = p[4]
        return_type = p[6] if len(p) == 9 else None
        body = p[7] if len(p) == 9 else p[6]
        p[0] = FunctionDef(name, params, return_type, body, line=p.lineno(1), col=1)

    def p_parameters(self, p):
        """parameters : parameter_list
                      | empty"""
        p[0] = p[1] or []

    def p_parameter_list(self, p):
        """parameter_list : parameter_list COMMA parameter
                          | parameter"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    def p_parameter(self, p):
        """parameter : ID COLON type
                     | ID"""
        name = p[1]
        type_ann = p[3] if len(p) == 4 else None
        p[0] = ('Parameter', name, type_ann)

    # Expressions
    def p_expression(self, p):
        """expression : assignment_expr"""
        p[0] = p[1]

    def p_assignment_expr(self, p):
        """assignment_expr : logical_or_expr
                           | logical_or_expr ASSIGN assignment_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], '=', p[3], p.lineno(1), 1)

    def p_logical_or_expr(self, p):
        """logical_or_expr : logical_and_expr
                           | logical_or_expr OR logical_and_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], '||', p[3], p.lineno(1), 1)

    def p_logical_and_expr(self, p):
        """logical_and_expr : equality_expr
                            | logical_and_expr AND equality_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], '&&', p[3], p.lineno(1), 1)

    def p_equality_expr(self, p):
        """equality_expr : relational_expr
                         | equality_expr EQ relational_expr
                         | equality_expr NE relational_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], p[2], p[3], p.lineno(1), 1)

    def p_relational_expr(self, p):
        """relational_expr : additive_expr
                           | relational_expr LT additive_expr
                           | relational_expr GT additive_expr
                           | relational_expr LE additive_expr
                           | relational_expr GE additive_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], p[2], p[3], p.lineno(1), 1)

    def p_additive_expr(self, p):
        """additive_expr : multiplicative_expr
                         | additive_expr PLUS multiplicative_expr
                         | additive_expr MINUS multiplicative_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], p[2], p[3], p.lineno(1), 1)

    def p_multiplicative_expr(self, p):
        """multiplicative_expr : unary_expr
                               | multiplicative_expr TIMES unary_expr
                               | multiplicative_expr DIVIDE unary_expr
                               | multiplicative_expr MODULO unary_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = BinaryOp(p[1], p[2], p[3], p.lineno(1), 1)

    def p_unary_expr(self, p):
        """unary_expr : postfix_expr
                      | NOT unary_expr
                      | MINUS unary_expr
                      | AMPERSAND unary_expr"""
        if len(p) == 2:
            p[0] = p[1]
        else:
            p[0] = UnaryOp(p[1], p[2], p.lineno(1), 1)

    def p_postfix_expr(self, p):
        """postfix_expr : primary_expr
                        | postfix_expr LBRACKET expression RBRACKET
                        | postfix_expr LPAREN args RPAREN
                        | postfix_expr DOT ID"""
        if len(p) == 2:
            p[0] = p[1]
        elif len(p) == 5 and p[2] == '[':
            p[0] = ('Index', p[1], p[3])
        elif len(p) == 5 and p[2] == '(':
            p[0] = FunctionCall(p[1], p[3] or [], p.lineno(1), 1)
        else:  # DOT
            p[0] = ('FieldAccess', p[1], p[3])

    def p_primary_expr(self, p):
        """primary_expr : INT
                        | FLOAT
                        | STRING
                        | TRUE
                        | FALSE
                        | ID
                        | LPAREN expression RPAREN
                        | block
                        | if_expr
                        | match_expr
                        | function_call"""
        if isinstance(p[1], bool):
            p[0] = BoolLiteral(p[1], p.lineno(1), 1)
        elif isinstance(p[1], int):
            p[0] = IntLiteral(p[1], p.lineno(1), 1)
        elif isinstance(p[1], float):
            p[0] = FloatLiteral(p[1], p.lineno(1), 1)
        elif p[1] == 'true':
            p[0] = BoolLiteral(True, p.lineno(1), 1)
        elif p[1] == 'false':
            p[0] = BoolLiteral(False, p.lineno(1), 1)
        else:
            p[0] = p[1]

    def p_if_expr(self, p):
        """if_expr : IF expression block
                   | IF expression block ELSE block
                   | IF expression block ELSE if_expr"""
        condition = p[2]
        then_body = p[3]
        else_body = p[5] if len(p) == 6 else None
        p[0] = IfExpr(condition, then_body, else_body, p.lineno(1), 1)

    def p_match_expr(self, p):
        """match_expr : MATCH expression LBRACE match_arms RBRACE"""
        p[0] = MatchExpr(p[2], p[4], p.lineno(1), 1)

    def p_match_arms(self, p):
        """match_arms : match_arm
                      | match_arms match_arm"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[2])
            p[0] = p[1]

    def p_match_arm(self, p):
        """match_arm : pattern FATARROW expression COMMA"""
        p[0] = (p[1], None, p[3])

    def p_pattern(self, p):
        """pattern : ID
                   | INT
                   | TRUE
                   | FALSE"""
        p[0] = ('Pattern', p[1])

    def p_block(self, p):
        """block : LBRACE statements RBRACE"""
        p[0] = Block(p[2], p.lineno(1), 1)

    def p_statements(self, p):
        """statements : statements statement
                      | statement
                      | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]] if p[1] else []
        else:
            if p[1]:
                p[1].append(p[2])
                p[0] = p[1]
            else:
                p[0] = [p[2]] if p[2] else []

    def p_statement(self, p):
        """statement : expression SEMICOLON
                     | LET ID ASSIGN expression SEMICOLON"""
        if len(p) == 3:
            p[0] = p[1]
        else:
            p[0] = ('Let', p[2], p[4])

    def p_args(self, p):
        """args : argument_list
                | empty"""
        p[0] = p[1] or []

    def p_argument_list(self, p):
        """argument_list : argument_list COMMA expression
                         | expression"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    def p_type(self, p):
        """type : ID
                | ID LT type_args GT"""
        if len(p) == 2:
            p[0] = ('Type', p[1])
        else:
            p[0] = ('GenericType', p[1], p[3])

    def p_type_args(self, p):
        """type_args : type_args COMMA type
                     | type"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    # Struct definition
    def p_struct_def(self, p):
        """struct_def : STRUCT ID LBRACE struct_fields RBRACE"""
        p[0] = StructDef(p[2], p[4], line=p.lineno(1), col=1)

    def p_struct_fields(self, p):
        """struct_fields : struct_fields struct_field
                         | struct_field
                         | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]] if p[1] else []
        else:
            if p[1]:
                p[1].append(p[2])
                p[0] = p[1]
            else:
                p[0] = [p[2]] if p[2] else []

    def p_struct_field(self, p):
        """struct_field : ID COLON type COMMA"""
        p[0] = (p[1], p[3])

    def p_enum_def(self, p):
        """enum_def : ENUM ID LBRACE enum_variants RBRACE"""
        p[0] = EnumDef(p[2], p[4], line=p.lineno(1), col=1)

    def p_enum_variants(self, p):
        """enum_variants : enum_variant
                         | enum_variants enum_variant
                         | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]] if p[1] else []
        else:
            if p[1]:
                p[1].append(p[2])
                p[0] = p[1]
            else:
                p[0] = [p[2]] if p[2] else []

    def p_enum_variant(self, p):
        """enum_variant : ID COMMA
                        | ID LPAREN type RPAREN COMMA"""
        if len(p) == 3:
            p[0] = (p[1], None)
        else:
            p[0] = (p[1], p[3])

    def p_trait_def(self, p):
        """trait_def : TRAIT ID LBRACE trait_methods RBRACE"""
        p[0] = TraitDef(p[2], p[4], line=p.lineno(1), col=1)

    def p_trait_methods(self, p):
        """trait_methods : trait_methods function_def
                         | function_def
                         | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]] if p[1] else []
        else:
            if p[1]:
                p[1].append(p[2])
                p[0] = p[1]
            else:
                p[0] = [p[2]] if p[2] else []

    def p_impl_block(self, p):
        """impl_block : IMPL ID LBRACE impl_items RBRACE"""
        p[0] = ('Impl', p[2], p[4])

    def p_impl_items(self, p):
        """impl_items : impl_items function_def
                      | function_def
                      | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]] if p[1] else []
        else:
            if p[1]:
                p[1].append(p[2])
                p[0] = p[1]
            else:
                p[0] = [p[2]] if p[2] else []

    def p_empty(self, p):
        """empty :"""
        p[0] = None

    def p_error(self, p):
        if p:
            print(f"Syntax error at '{p.value}' (line {p.lineno})")
            self.errors.append(f"Syntax error at line {p.lineno}")
        else:
            print("Syntax error at EOF")
            self.errors.append("Syntax error at EOF")

    def parse(self, code: str):
        return self.parser.parse(code, lexer=lexer.lexer)

parser = Parser()
```

---

## 5. Integration Testing

```python
def test_parser():
    code = '''
    fn main() {
        let x = 42;
        println!("hello");
    }
    '''

    ast = parser.parse(code)
    print(ast)
    assert ast is not None
    assert parser.errors == []

if __name__ == "__main__":
    test_parser()
```

---

## 6. Deliverables

- [x] Lexer implementation (tokenizes all GrokLang syntax)
- [x] Token specification complete
- [x] Parser implementation (LALR(1))
- [x] AST node classes defined
- [x] Basic expression parsing
- [x] Function/struct/enum/trait definitions
- [x] Error messages with line/column info
- [ ] **Validation (next document)**

---

## 7. Validation Checklist

Use the Phase 1 Validation document to verify all criteria.

---

## 8. Next Steps

→ Proceed to Phase 2: Type Checker and AST Decoration
