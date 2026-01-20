class MacroExpander:
    def __init__(self):
        self.macros = {}

    def define_macro(self, name: str, rules):
        """Store macro definition"""
        self.macros[name] = rules

    def expand_macro(self, name: str, args):
        """Expand macro call with pattern matching"""
        if name not in self.macros:
            raise ValueError(f"Macro {name} not defined")

        rules = self.macros[name]
        for pattern, template in rules:
            if self._matches_pattern(pattern, args):
                return self._substitute_template(template, pattern, args)

        raise ValueError(f"No matching rule for macro {name} with args {args}")

    def _matches_pattern(self, pattern, args):
        """Check if args match the pattern"""
        # Simple: check length and types
        if isinstance(pattern, list) and len(pattern) == len(args):
            return True
        return len(pattern) == len(args) if hasattr(pattern, '__len__') else True

    def _substitute_template(self, template, pattern, args):
        """Substitute pattern variables in template"""
        if isinstance(template, str):
            # Map pattern vars to args
            for i, var in enumerate(pattern):
                if i < len(args):
                    template = template.replace(f'${var}', str(args[i]))
            return template
        elif isinstance(template, tuple):
            return tuple(self._substitute_template(t, pattern, args) for t in template)
        return template

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