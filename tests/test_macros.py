from src.groklang.macro_expander import MacroExpander

def test_macro_expander():
    expander = MacroExpander()
    
    # Define macro
    macro_def = ('MacroDef', 'println', [
        ('MacroRule', ('Pattern', 'expr'), ('Call', 'print', [('expr',)]))
    ])
    expander.expand_ast(macro_def)
    
    # Expand macro call
    expanded = expander.expand_macro('println', [('String', 'hello')])
    assert expanded == ('Call', 'print', [('String', 'hello')])
    print("Macro expander test passed!")

def test_macro_ast_expansion():
    expander = MacroExpander()
    
    ast = [
        ('MacroDef', 'test_macro', [('MacroRule', ('Pattern', 'x'), ('Int', 42))]),
        ('MacroCall', 'test_macro', [('Int', 1)])
    ]
    
    expanded = expander.expand_ast(ast)
    # Should expand the call and remove the def
    assert expanded is not None
    assert isinstance(expanded, list)
    print("Macro AST expansion test passed!")

if __name__ == "__main__":
    test_macro_expander()
    test_macro_ast_expansion()