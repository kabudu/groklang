from src.groklang.module_system import ModuleResolver, PrivacyChecker

def test_module_definition():
    resolver = ModuleResolver()
    
    # Mock items as tuples
    items = [
        ('FunctionDef', 'pub_func', [], None, [], 'pub', 0, 1),
        ('FunctionDef', 'priv_func', [], None, [], None, 0, 1)
    ]
    
    resolver.define_module('test_mod', items)
    
    assert 'test_mod' in resolver.modules
    assert 'pub_func' in resolver.modules['test_mod']['public']
    assert 'priv_func' not in resolver.modules['test_mod']['public']
    print("Module definition test passed!")

def test_privacy_checking():
    resolver = ModuleResolver()
    checker = PrivacyChecker(resolver)
    
    # Define module with public item
    items = [('FunctionDef', 'pub_func', [], None, [], 'pub', 0, 1)]
    resolver.define_module('test_mod', items)
    
    # Valid use
    ast = [('Use', ['test_mod'])]
    errors = checker.check_privacy(ast)
    assert len(errors) == 0
    print("Privacy checking test passed!")

def test_module_resolution():
    resolver = ModuleResolver()
    
    items = [('FunctionDef', 'func', [], None, [], 'pub', 0, 1)]
    resolver.define_module('mod', items)
    
    imported = resolver.resolve_use(['mod'], 'other')
    assert 'func' in imported
    
    # Check access
    assert resolver.can_access('mod', 'func', 'other')  # public
    assert not resolver.can_access('mod', 'private', 'other')  # not exists
    print("Module resolution test passed!")

if __name__ == "__main__":
    test_module_definition()
    test_privacy_checking()
    test_module_resolution()