# GrokLang Syntax and Grammar Specification

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Format**: Extended Backus-Naur Form (EBNF)

---

## 1. Lexical Tokens

### 1.1 Keywords

```
Reserved keywords (context-sensitive):
fn      let     mut     if      else    match
for     while   loop    break   continue
return  struct  enum    trait   impl    use
mod     pub     pub(crate)
actor   spawn   send    receive
true    false   self    Self
where   type    as      unsafe
extern  const   static  macro
ai      ai_optimize    ai_test    ai_translate
```

### 1.2 Operators and Delimiters

```
Arithmetic:       +  -  *  /  %
Comparison:       ==  !=  <  >  <=  >=
Logical:          &&  ||  !
Bitwise:          &  |  ^  ~  <<  >>
Assignment:       =  +=  -=  *=  /=  %=
Other:            =>  ->  ::  ..  ...  ?

Delimiters:       ( ) [ ] { } , ; : . @ # $
```

### 1.3 Identifiers

```
identifier = letter { letter | digit | underscore }
letter     = 'a'..'z' | 'A'..'Z' | '_'
digit      = '0'..'9'

// Examples:
foo
_private
MyStruct
x1
___

// NOT valid:
1abc        // Starts with digit
my-var      // Hyphen not allowed
hello!      // Exclamation mark (reserved)
```

### 1.4 Literals

```
// Integer literals
decimal_int    = digit { digit }              // 42, 1000
hex_int        = '0x' hex_digit { hex_digit } // 0xFF, 0xDEADBEEF
binary_int     = '0b' ('0'|'1') { ('0'|'1') }// 0b1010
octal_int      = '0o' octal_digit { octal_digit } // 0o755

// Integer with suffix
int_literal    = int_part type_suffix?
type_suffix    = 'i8' | 'i16' | 'i32' | 'i64' | 'i128' | 'isize'
                | 'u8' | 'u16' | 'u32' | 'u64' | 'u128' | 'usize'

Examples: 42i32, 1000u64, 0xFF_usize

// Float literals
float_literal  = digit { digit } '.' digit { digit } exponent?
                | digit { digit } exponent
exponent       = ('e' | 'E') ['+' | '-'] digit { digit }

type_suffix    = 'f32' | 'f64'

Examples: 3.14, 1.0e-10, 42.0f64

// String literals
string_literal = '"' { string_char | escape_seq } '"'
escape_seq     = '\' ('n' | 't' | 'r' | '"' | '\' | '0' | 'x' hex_digit hex_digit | 'u' '{' hex_digit+ '}')

Examples: "hello", "line1\nline2", "unicode: \u{1F600}"

// Character literal
char_literal   = ''' ( char | escape_seq ) '''

Examples: 'a', '\n', '\u{1F600}'

// Raw string (no escapes)
raw_string     = 'r' '"' { any_char } '"'

Example: r"C:\path\to\file"

// Boolean literals
bool_literal   = 'true' | 'false'
```

### 1.5 Comments

```
// Single-line comment (extends to newline)
/* Multi-line comment /* can be nested */ */

/// Documentation comment (on next item)
//! Documentation comment (on current item)
```

---

## 2. Expressions (EBNF Grammar)

```ebnf
program = { item } ;

item = function_def | struct_def | enum_def | trait_def | impl_block
      | decorator item
      | use_statement
      | module_def ;

use_statement = 'use' path { '::' path } [ '::' '{' imports '}' ] ';' ;
imports = import_item { ',' import_item } [','] ;
import_item = identifier [ 'as' identifier ] ;

// ============ Functions ============

function_def = [ decorator ] 'fn' identifier [ type_params ]
               '(' [ parameters ] ')' [ '->' type ] block ;

type_params = '<' type_param { ',' type_param } [','] '>' ;
type_param = identifier [ ':' trait_bounds ]
            | 'const' identifier ':' type ;

trait_bounds = trait_type { '+' trait_type } ;
trait_type = path [ '<' type_args '>' ] ;

parameters = parameter { ',' parameter } [','] ;
parameter = [ 'mut' ] pattern [ ':' type ] [ '=' expression ] ;

block = '{' { statement } [ expression ] '}' ;

// ============ Statements ============

statement = let_binding ';'
          | expression ';'
          | item ;

let_binding = 'let' [ 'mut' ] pattern [ ':' type ] '=' expression ;

pattern = identifier
        | '_'
        | literal
        | '(' pattern { ',' pattern } [','] ')'
        | path '{' field_patterns '}'
        | path '(' [ pattern { ',' pattern } [','] ] ')'
        | pattern '|' pattern
        | pattern [ 'if' expression ] ;

field_patterns = field_pattern { ',' field_pattern } [','] ;
field_pattern = identifier [ ':' pattern ] ;

// ============ Expressions ============

expression = assignment_expr ;

assignment_expr = logical_or_expr [ assignment_op assignment_expr ] ;
assignment_op = '=' | '+=' | '-=' | '*=' | '/=' | '%=' | '&=' | '|=' | '^=' | '<<=' | '>>=' ;

logical_or_expr = logical_and_expr { '||' logical_and_expr } ;
logical_and_expr = bitwise_or_expr { '&&' bitwise_or_expr } ;

bitwise_or_expr = bitwise_xor_expr { '|' bitwise_xor_expr } ;
bitwise_xor_expr = bitwise_and_expr { '^' bitwise_and_expr } ;
bitwise_and_expr = equality_expr { '&' equality_expr } ;

equality_expr = relational_expr { ( '==' | '!=' ) relational_expr } ;

relational_expr = shift_expr { ( '<' | '>' | '<=' | '>=' ) shift_expr } ;

shift_expr = additive_expr { ( '<<' | '>>' ) additive_expr } ;

additive_expr = multiplicative_expr { ( '+' | '-' ) multiplicative_expr } ;

multiplicative_expr = unary_expr { ( '*' | '/' | '%' ) unary_expr } ;

unary_expr = [ unary_op ] postfix_expr ;
unary_op = '!' | '-' | '*' | '&' | '&mut' ;

postfix_expr = primary_expr { postfix_op } ;
postfix_op = '.' identifier [ '(' [ args ] ')' ]  // Method call
           | '.' identifier                       // Field access
           | '[' expression ']'                   // Indexing
           | '(' [ args ] ')'                     // Function call
           | '?' ;                                // Try operator

primary_expr = literal
             | identifier
             | path '::' identifier
             | 'self'
             | 'Self'
             | '(' [ expression ] ')'
             | block
             | if_expr
             | match_expr
             | loop_expr
             | for_expr
             | while_expr
             | closure_expr
             | array_expr
             | tuple_expr
             | struct_literal
             | enum_literal
             | spawn_expr
             | return_expr
             | break_expr
             | continue_expr
             | macro_call ;

// ============ Control Flow ============

if_expr = 'if' expression block { 'else if' expression block } [ 'else' block ] ;

match_expr = 'match' expression '{' { match_arm } '}' ;
match_arm = pattern [ 'if' expression ] '=>' ( expression | block ) [','] ;

loop_expr = 'loop' block ;

for_expr = 'for' pattern 'in' expression block ;

while_expr = 'while' expression block ;

return_expr = 'return' [ expression ] ;
break_expr = 'break' [ identifier ] [ expression ] ;
continue_expr = 'continue' [ identifier ] ;

// ============ Closures ============

closure_expr = [ '|' [ parameters ] '|' ] block
             | [ 'move' ] '|' [ parameters ] '|' '->' type block
             | [ 'move' ] '|' [ parameters ] '|' expression ;

// ============ Collections ============

array_expr = '[' [ expression { ',' expression } [','] ] ']' ;

tuple_expr = '(' expression ',' { expression } [','] ')' ;

// ============ Structs and Enums ============

struct_literal = path '{' [ field_inits ] [ ',' '..' expression ] '}' ;
field_inits = field_init { ',' field_init } [','] ;
field_init = identifier [ ':' expression ] ;

enum_literal = path '::' identifier [ '(' [ args ] ')' ]
             | path '{' identifier [ ':' expression ] '}' ;

// ============ Spawning and Concurrency ============

spawn_expr = 'spawn' block ;

// ============ Macros ============

macro_call = identifier '!' [ '(' macro_args ')' | '[' macro_args ']' | '{' macro_args '}' ] ;
macro_args = token { token } ;  // Simplified; actual macro syntax more complex

// ============ Decorator ============

decorator = '#' '[' path [ '(' decorator_args ')' ] ']' ;
decorator_args = identifier { ',' identifier } ;

// ============ Arguments ============

args = expression { ',' expression } [','] ;

// ============ Types ============

type = primary_type [ postfix_type ] ;

primary_type = identifier
             | path '<' type_args '>'
             | '(' [ type { ',' type } [','] ] ')'
             | '[' type [ ';' expression ] ']'
             | '&' [ lifetime ] [ 'mut' ] type
             | '*' [ 'const' | 'mut' ] type
             | 'dyn' trait_bounds
             | 'fn' '(' [ type { ',' type } [','] ] ')' '->' type ;

postfix_type = { '&' | '*' } ;

type_args = type { ',' type } [',']
          | type_arg { ',' type_arg } [','] ;

type_arg = identifier '=' type
         | type ;

# ============ Additional Expressions and Constructs ============

decorator = '#' '[' path [ '(' decorator_args ')' ] ']' ;
decorator_args = identifier { ',' identifier } ;

use_statement = 'use' path { '::' path } [ '::' '{' imports '}' ] ';' ;
imports = import_item { ',' import_item } [','] ;
import_item = identifier [ 'as' identifier ] ;

module_def = 'mod' identifier '{' { item } '}' ;

closure_expr = [ '|' [ parameters ] '|' ] block
              | [ 'move' ] '|' [ parameters ] '|' '->' type block
              | [ 'move' ] '|' [ parameters ] '|' expression ;

array_expr = '[' [ expression { ',' expression } [','] ] ']' ;

tuple_expr = '(' expression ',' { expression } [','] ')' ;

struct_literal = path '{' [ field_inits ] [ ',' '..' expression ] '}' ;
field_inits = field_init { ',' field_init } [','] ;
field_init = identifier [ ':' expression ] ;

enum_literal = path '::' identifier [ '(' [ args ] ')' ]
              | path '{' identifier [ ':' expression ] '}' ;

spawn_expr = 'spawn' block ;

loop_expr = 'loop' block ;

for_expr = 'for' pattern 'in' expression block ;

while_expr = 'while' expression block ;

return_expr = 'return' [ expression ] ;

break_expr = 'break' [ identifier ] [ expression ] ;

continue_expr = 'continue' [ identifier ] ;

macro_call = identifier '!' [ '(' macro_args ')' | '[' macro_args ']' | '{' macro_args '}' ] ;
macro_args = token { token } ;  // Simplified

pattern = identifier
        | '_'
        | literal
        | '(' pattern { ',' pattern } [','] ')'
        | path '{' field_patterns '}'
        | path '(' [ pattern { ',' pattern } [','] ] ')'
        | pattern '|' pattern
        | pattern [ 'if' expression ] ;

field_patterns = field_pattern { ',' field_pattern } [','] ;
field_pattern = identifier [ ':' pattern ] ;

primary_type = identifier
              | path '<' type_args '>'
              | '(' [ type { ',' type } [','] ] ')'
              | '[' type [ ';' expression ] ']'
              | '&' [ 'mut' ] type
              | '*' [ 'const' | 'mut' ] type
              | 'dyn' trait_bounds
              | 'fn' '(' [ type { ',' type } [','] ] ')' '->' type ;

postfix_type = { '&' | '*' } ;
```

### 2.1 Updated Keywords

```python
tokens = (
    # Literals
    'INT', 'FLOAT', 'STRING', 'CHAR', 'TRUE', 'FALSE',

    # Identifiers and keywords
    'ID',

    # Keywords (reserved)
    'FN', 'LET', 'MUT', 'IF', 'ELSE', 'MATCH',
    'FOR', 'WHILE', 'LOOP', 'BREAK', 'CONTINUE',
    'RETURN', 'STRUCT', 'ENUM', 'TRAIT', 'IMPL', 'USE',
    'MOD', 'PUB', 'SELF', 'SELFTYPE',
    'WHERE', 'TYPE', 'AS', 'UNSAFE',
    'EXTERN', 'CONST', 'STATIC', 'MACRO',
    'AI', 'ACTOR', 'SPAWN',

    # Operators
    'PLUS', 'MINUS', 'TIMES', 'DIVIDE', 'MODULO',
    'EQ', 'NE', 'LT', 'GT', 'LE', 'GE',
    'AND', 'OR', 'NOT',
    'AMPERSAND', 'PIPE', 'CARET', 'TILDE',
    'LSHIFT', 'RSHIFT',
    'ASSIGN', 'PLUSASSIGN', 'MINUSASSIGN', 'TIMESASSIGN', 'DIVIDEASSIGN',
    'LPAREN', 'RPAREN', 'LBRACE', 'RBRACE', 'LBRACKET', 'RBRACKET',
    'COMMA', 'SEMICOLON', 'COLON', 'DOUBLECOLON', 'DOT', 'ARROW', 'FATARROW',
    'ELLIPSIS', 'QUESTION', 'AT', 'HASH', 'DOLLAR',
)
```

### 4.2 Token Rules (PLY)

```python
# Keywords (defined before ID to have precedence)
reserved = {
    'fn': 'FN',
    'let': 'LET',
    'mut': 'MUT',
    'if': 'IF',
    'else': 'ELSE',
    'match': 'MATCH',
    'for': 'FOR',
    'while': 'WHILE',
    'loop': 'LOOP',
    'break': 'BREAK',
    'continue': 'CONTINUE',
    'return': 'RETURN',
    'struct': 'STRUCT',
    'enum': 'ENUM',
    'trait': 'TRAIT',
    'impl': 'IMPL',
    'use': 'USE',
    'mod': 'MOD',
    'pub': 'PUB',
    'self': 'SELF',
    'Self': 'SELFTYPE',
    'true': 'TRUE',
    'false': 'FALSE',
    'where': 'WHERE',
    'type': 'TYPE',
    'as': 'AS',
    'unsafe': 'UNSAFE',
    'extern': 'EXTERN',
    'const': 'CONST',
    'static': 'STATIC',
    'macro': 'MACRO',
    'ai': 'AI',
    'actor': 'ACTOR',
    'spawn': 'SPAWN',
}

def t_ID(t):
    r'[a-zA-Z_][a-zA-Z_0-9]*'
    t.type = reserved.get(t.value, 'ID')
    return t

# Floating point number
def t_FLOAT(t):
    r'(\d+\.\d*|\d*\.\d+)([eE][+-]?\d+)?[fF]?|(\d+)[eE][+-]?\d+[fF]?'
    t.value = float(t.value)
    return t

# Integer (multiple bases)
def t_INT(t):
    r'(0[xX][0-9a-fA-F]+|0[bB][01]+|0[oO][0-7]+|\d+)[uU]?[iI]?(8|16|32|64|128)?'
    t.value = int(t.value.rstrip('uUiI'), 0)
    return t

# String literal
def t_STRING(t):
    r'"([^"\\]|\\.)*"'
    t.value = t.value[1:-1]  # Remove quotes
    return t

# Character literal
def t_CHAR(t):
    r"'([^'\\]|\\.)?'"
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

# Single-character operators
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

# Comments (ignored)
def t_COMMENT(t):
    r'//[^\n]*'
    pass  # Ignore

def t_MCOMMENT(t):
    r'/\*(.|\n)*?\*/'
    t.lexer.lineno += t.value.count('\n')
    pass  # Ignore

# Newlines (tracked for line numbers)
def t_newline(t):
    r'\n+'
    t.lexer.lineno += len(t.value)

# Ignored characters (whitespace)
t_ignore = ' \t\r'

# Error handling
def t_error(t):
    print(f"Illegal character '{t.value[0]}'")
    t.lexer.skip(1)
```

---

## 5. Concrete Syntax Examples

### 5.1 Function Definition

```groklang
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// With type inference
fn add(a, b) {
    a + b
}

// With decorators
#[ai_optimize]
fn compute(x: f64) -> f64 {
    x * x + 2.0 * x + 1.0
}
```

### 5.2 Struct Definition

```groklang
struct Point {
    x: f64,
    y: f64,
}

struct Generic<T> {
    value: T,
    next: Option<Box<Generic<T>>>,
}

// Generic with trait bounds
struct Container<T : Clone> {
    items: Vec<T>,
}
```

### 5.3 Trait Definition

```groklang
trait Iterator {
    type Item;
    fn next(mut self) -> Option<Self::Item>;
    fn skip(mut self, n: usize) -> ();
}

trait Clone {
    fn clone(self) -> Self;
}
```

### 5.4 Pattern Matching

```groklang
match result {
    Ok(value) => println!("success: {}", value),
    Err(e) => println!("error: {}", e),
}

match (x, y) {
    (0, 0) => println!("origin"),
    (x, 0) => println!("on x-axis"),
    (0, y) => println!("on y-axis"),
    (x, y) if x == y => println!("diagonal"),
    _ => println!("general point"),
}
```

### 5.5 Closures

```groklang
let double = |x| x * 2;

let add = |x, y| {
    let sum = x + y;
    sum
};

let captured = {
    let factor = 10;
    |x| x * factor
};
```

---

## 6. Parser Implementation Notes (PLY compatible)

The parser should be implemented as a bottom-up LALR(1) parser using PLY.

**Key rules**:

1. **Precedence resolution**: Use PLY's precedence declarations
2. **Associativity**: Specify right/left as needed
3. **Shift/reduce conflicts**: Resolve via precedence
4. **AST construction**: Each grammar rule returns an AST node
5. **Error recovery**: Skip to next statement on parse error

**Validation pass** (after parsing):

- Check for duplicate identifiers in same scope
- Verify all referenced items exist
- Validate decorator syntax

---

## 7. Validation Criteria

- [ ] Lexer correctly tokenizes all GrokLang constructs
- [ ] Parser builds correct AST from all grammar rules
- [ ] Operator precedence matches specification
- [ ] All keywords properly reserved
- [ ] Comments correctly ignored
- [ ] String/character escapes handled properly
- [ ] Generic syntax validated
- [ ] Decorator syntax validated
- [ ] Error messages point to correct locations

---

## 8. Related Documents

- [01-Architecture-Overview.md](01-Architecture-Overview.md) - System architecture
- [02-Type-System-Specification.md](02-Type-System-Specification.md) - Type system details
- [04-Runtime-Memory-Model.md](04-Runtime-Memory-Model.md) - Runtime semantics
