import ply.yacc as yacc
from . import lexer  # Import lexer module
from .lexer import tokens
from .ast_nodes import *

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

    def p_expr(self, p):
        """expr : assignment_expr
                  | if_expr
                  | match_expr
                  | let_statement
                  | return_expr
                  | break_expr
                  | continue_expr
                  | block"""
        p[0] = p[1]

    def p_if_expr(self, p):
        """if_expr : IF expression block
                    | IF expression block ELSE block
                    | IF expression block ELSE if_expr"""
        if len(p) == 4:
            p[0] = IfExpr(p[2], p[3], None, p.lineno(1), 1)
        elif len(p) == 6:
            p[0] = IfExpr(p[2], p[3], p[5], p.lineno(1), 1)
        else:
            p[0] = p[5]  # else if

    def p_match_expr(self, p):
        """match_expr : MATCH expression LBRACE match_arms RBRACE"""
        p[0] = MatchExpr(p[2], p[4], p.lineno(1), 1)

    def p_match_arms(self, p):
        """match_arms : match_arm
                       | match_arms COMMA match_arm"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    def p_match_arm(self, p):
        """match_arm : pattern FATARROW expression"""
        p[0] = (p[1], p[3])

    def p_let_statement(self, p):
        """let_statement : LET ID ASSIGN expression
                          | LET MUT ID ASSIGN expression"""
        if len(p) == 5:
            p[0] = LetStmt(p[2], False, p[4], p.lineno(1), 1)
        else:
            p[0] = LetStmt(p[3], True, p[5], p.lineno(1), 1)

    def p_block(self, p):
        """block : LBRACE statements RBRACE"""
        p[0] = Block(p[2], p.lineno(1), 1)

    def p_statements(self, p):
        """statements : 
                       | statements expr SEMICOLON"""
        if len(p) == 1:
            p[0] = []
        else:
            p[1].append(p[2])
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
                       | BANG unary_expr
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

    # Decorator
    def p_decorator(self, p):
        """decorator : HASH LBRACKET ID LPAREN decorator_args RPAREN RBRACKET
                      | HASH LBRACKET ID RBRACKET"""
        if len(p) == 5:
            p[0] = ('Decorator', p[3], [])
        else:
            p[0] = ('Decorator', p[3], p[5])

    def p_decorator_args(self, p):
        """decorator_args : ID
                           | decorator_args COMMA ID"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    # Update item to include decorators and use
    def p_item(self, p):
        """item : visibility item
                | decorator item
                | function_def
                | struct_def
                | enum_def
                | trait_def
                | impl_block
                | use_statement
                | module_def"""
        if len(p) == 3 and p[1] == 'pub':  # visibility item
            item = p[2]
            item.visibility = 'pub'
            p[0] = item
        elif len(p) == 3:  # decorator item
            item = p[2]
            item.decorators.append(p[1][1])  # Add decorator name
            p[0] = item
        else:
            p[0] = p[1]

    def p_visibility(self, p):
        """visibility : PUB"""
        p[0] = p[1]

    # Use statement
    def p_use_statement(self, p):
        """use_statement : USE path SEMICOLON"""
        p[0] = ('Use', p[2])

    def p_path(self, p):
        """path : ID
                | path DOUBLECOLON ID"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    # Module definition
    def p_module_def(self, p):
        """module_def : MOD ID LBRACE items RBRACE"""
        p[0] = ('Module', p[2], p[4])

    # Update primary_expr for new constructs
    def p_primary_expr(self, p):
        """primary_expr : INT
                         | FLOAT
                         | STRING
                         | RAW_STRING
                         | BYTE_STRING
                         | TRUE
                         | FALSE
                         | ID
                         | SELF
                         | SELFTYPE
                         | LPAREN expression RPAREN
                         | block
                         | if_expr
                         | match_expr
                         | loop_expr
                         | closure_expr
                         | array_expr
                         | tuple_expr
                         | struct_literal
                         | enum_literal
                         | spawn_expr
                          | return_expr
                          | break_expr
                          | continue_expr"""
        if p[1] in ('true', 'false'):
            p[0] = BoolLiteral(p[1] == 'true', p.lineno(1), 1)
        elif isinstance(p[1], (int, float)):
            if isinstance(p[1], int):
                p[0] = IntLiteral(p[1], p.lineno(1), 1)
            else:
                p[0] = FloatLiteral(p[1], p.lineno(1), 1)
        elif isinstance(p[1], str):
            p[0] = StringLiteral(p[1], p.lineno(1), 1)
        elif isinstance(p[1], bytes):
            p[0] = ByteStringLiteral(p[1], p.lineno(1), 1)
        elif p[1] == 'self':
            p[0] = Identifier('self', p.lineno(1), 1)
        elif p[1] == 'Self':
            p[0] = Identifier('Self', p.lineno(1), 1)
        else:
            p[0] = p[1]

    # Array and tuple expressions
    def p_array_expr(self, p):
        """array_expr : LBRACKET array_elements RBRACKET"""
        p[0] = ('Array', p[2])

    def p_array_elements(self, p):
        """array_elements : expression
                           | array_elements COMMA expression
                           | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    def p_tuple_expr(self, p):
        """tuple_expr : LPAREN tuple_elements RPAREN"""
        p[0] = ('Tuple', p[2])

    def p_tuple_elements(self, p):
        """tuple_elements : expression COMMA expression
                           | tuple_elements COMMA expression"""
        if len(p) == 4:
            p[0] = [p[1], p[3]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    # Struct and enum literals
    def p_struct_literal(self, p):
        """struct_literal : ID LBRACE field_inits RBRACE"""
        p[0] = ('StructLiteral', p[1], p[3])

    def p_field_inits(self, p):
        """field_inits : field_init
                         | field_inits COMMA field_init
                         | empty"""
        if p[1] is None:
            p[0] = {}
        elif len(p) == 2:
            p[0] = {p[1][0]: p[1][1]}
        else:
            p[1][p[3][0]] = p[3][1]

    def p_field_init(self, p):
        """field_init : ID COLON expression"""
        p[0] = (p[1], p[3])
            p[0] = p[1]


    def p_enum_literal(self, p):
        """enum_literal : ID DOUBLECOLON ID LPAREN expression RPAREN
                        | ID DOUBLECOLON ID LBRACE field_inits RBRACE"""
        if p[4] == '(':
            p[0] = ('EnumLiteral', p[1], p[3], p[5])
        else:
            p[0] = ('EnumLiteralStruct', p[1], p[3], p[5])

    # Closures
    def p_closure_expr(self, p):
        """closure_expr : PIPE parameters PIPE expression
                        | PIPE parameters PIPE ARROW type block
                        | PIPE PIPE expression"""
        if len(p) == 5:
            p[0] = ('Closure', p[2], None, p[4])
        elif len(p) == 7:
            p[0] = ('Closure', p[2], p[5], p[6])
        else:
            p[0] = ('Closure', [], None, p[3])

    # Loops and control flow
    def p_loop_expr(self, p):
        """loop_expr : LOOP block"""
        p[0] = ('Loop', p[2])



    # Expressions for return/break/continue
    def p_return_expr(self, p):
        """return_expr : RETURN expression
                       | RETURN"""
        p[0] = ('Return', p[2] if len(p) == 3 else None)

    def p_break_expr(self, p):
        """break_expr : BREAK
                      | BREAK expression"""
        p[0] = ('Break', p[2] if len(p) == 3 else None)

    def p_continue_expr(self, p):
        """continue_expr : CONTINUE"""
        p[0] = ('Continue',)

    # Spawn expression
    def p_spawn_expr(self, p):
        """spawn_expr : SPAWN block"""
        p[0] = ('Spawn', p[2])

    # Macros
    # Update patterns for more complex matching
    def p_pattern(self, p):
        """pattern : ID
                   | INT
                   | TRUE
                   | FALSE
                   | UNDERSCORE
                   | LPAREN pattern_list RPAREN
                   | ID LBRACE field_patterns RBRACE
                   | ID LPAREN pattern_list RPAREN"""
        if len(p) == 2:
            p[0] = ('Pattern', p[1])
        elif p[2] == '(':
            p[0] = ('TuplePattern', p[1], p[3])
        else:
            p[0] = ('StructPattern', p[1], p[3])

    def p_pattern_list(self, p):
        """pattern_list : pattern
                        | pattern_list COMMA pattern
                        | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

    def p_field_patterns(self, p):
        """field_patterns : field_pattern
                          | field_patterns COMMA field_pattern
                          | empty"""
        if p[1] is None:
            p[0] = {}
        elif len(p) == 2:
            p[0] = {p[1][0]: p[1][1]}
        else:
            p[1][p[3][0]] = p[3][1]
            p[0] = p[1]

    def p_field_pattern(self, p):
        """field_pattern : ID
                         | ID COLON pattern"""
        if len(p) == 2:
            p[0] = (p[1], ('Pattern', p[1]))
        else:
            p[0] = (p[1], p[3])

    # Update type to include more
    def p_type(self, p):
        """type : ID
                | ID LT type_args GT
                | LPAREN type_list RPAREN
                | LBRACKET type RBRACKET
                | AMPERSAND type
                | AMPERSAND MUT type
                | FN LPAREN type_list RPAREN ARROW type"""
        if len(p) == 2:
            if p[1] == 'Self':
                p[0] = ('Type', 'Self')
            else:
                p[0] = ('Type', p[1])
        elif p[2] == '<':
            p[0] = ('GenericType', p[1], p[3])
        elif p[1] == '(':
            p[0] = ('TupleType', p[2])
        elif p[1] == '[':
            p[0] = ('ArrayType', p[2])
        elif p[1] == '&':
            if len(p) == 3:
                p[0] = ('RefType', False, p[2])
            else:
                p[0] = ('RefType', True, p[3])
        else:  # fn
            p[0] = ('FnType', p[3], p[6])

    def p_type_list(self, p):
        """type_list : type
                     | type_list COMMA type
                     | empty"""
        if p[1] is None:
            p[0] = []
        elif len(p) == 2:
            p[0] = [p[1]]
        else:
            p[1].append(p[3])
            p[0] = p[1]

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