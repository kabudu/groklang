import ply.lex as lex

# Token list
tokens = (
    'INT', 'FLOAT', 'STRING', 'RAW_STRING', 'BYTE_STRING', 'CHAR', 'TRUE', 'FALSE', 'ID', 'UNDERSCORE',
    'FN', 'LET', 'MUT', 'IF', 'ELSE', 'MATCH', 'FOR', 'WHILE', 'LOOP', 'IN',
    'BREAK', 'CONTINUE', 'RETURN', 'STRUCT', 'ENUM', 'TRAIT', 'IMPL', 'USE',
    'MOD', 'PUB', 'SELF', 'SELFTYPE', 'WHERE', 'TYPE', 'AS', 'UNSAFE',
    'EXTERN', 'CONST', 'STATIC', 'MACRO', 'AI', 'ACTOR', 'SPAWN', 'MOVE',
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
    'macro': 'MACRO', 'macro_rules': 'MACRO_RULES', 'ai': 'AI', 'actor': 'ACTOR', 'spawn': 'SPAWN',
    'move': 'MOVE', 'in': 'IN', '_': 'UNDERSCORE',
}

# Token rules
def t_RAW_STRING(t):
    r'r"([^"])*"'
    t.value = t.value[2:-1]  # Remove r"
    return t

def t_BYTE_STRING(t):
    r'b"([^"\\\\]|\\\\.)*"'
    t.value = t.value[2:-1]  # Remove b"
    t.value = t.value.encode('utf-8')  # Convert to bytes
    return t

def t_ID(t):
    r'[a-zA-Z_][a-zA-Z_0-9]*'
    t.type = reserved.get(t.value, 'ID')
    return t

def t_FLOAT(t):
    r'(\d+\.\d+|\d*\.\d+)([eE][+-]?\d+)?[fF]?|(\d+)[eE][+-]?\d+[fF]?'
    t.value = float(t.value)
    return t

def t_INT(t):
    r'(0[xX][0-9a-fA-F]+|0[bB][01]+|0[oO][0-7]+|\d+)[uUiI]?(8|16|32|64|128)?'
    # Strip suffix before converting
    value_str = t.value
    if any(c in value_str for c in 'uUiI'):
        # Find the start of suffix
        for i, c in enumerate(value_str):
            if c in 'uUiI':
                value_str = value_str[:i]
                break
    t.value = int(value_str, 0)
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

lexer = lex.lex(debug=0, optimize=0)