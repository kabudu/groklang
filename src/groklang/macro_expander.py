class MacroExpander:
    def __init__(self):
        self.macros = {}

    def define_macro(self, name: str, rules):
        """Store macro definition"""
        self.macros[name] = rules

    def expand_macro(self, name: str, args):
        """Expand macro call"""
        if name not in self.macros:
            raise ValueError(f"Macro {name} not defined")

        rules = self.macros[name]
        # Simple expansion: return first rule's template with args substituted
        for rule in rules:
            pattern, template = rule[1], rule[2]
            # Basic substitution (placeholder)
            if isinstance(template, tuple) and len(template) > 2:
                return (template[0], template[1], args)
            return template

        raise ValueError(f"No matching rule for macro {name}")

    def expand_ast(self, ast):
        """Expand macros in AST"""
        if isinstance(ast, tuple):
            if ast[0] == 'MacroDef':
                self.define_macro(ast[1], ast[2])
                return None  # Remove definition
            elif ast[0] == 'MacroCall':
                return self.expand_macro(ast[1], ast[2])
            else:
                return tuple(self.expand_ast(item) for item in ast if item is not None)
        elif isinstance(ast, list):
            return [self.expand_ast(item) for item in ast if item is not None]
        else:
            return ast