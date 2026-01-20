# groklang/optimizer.py
"""GrokLang Compiler Optimizations"""

class Optimizer:
    def __init__(self, ast):
        self.ast = ast
        self.changed = False

    def optimize(self):
        """Apply all optimization passes with semantic preservation"""
        iterations = 0
        while iterations < 10 and self.changed:  # Limit iterations to prevent infinite loops
            self.changed = False
            self.constant_folding()
            self.dead_code_elimination()
            self.inline_functions()
            iterations += 1
        return self.ast

    def constant_folding(self):
        """Fold constant expressions while preserving semantics"""
        def fold_expr(expr):
            if hasattr(expr, 'left') and hasattr(expr, 'right') and hasattr(expr, 'op'):
                left = fold_expr(expr.left)
                right = fold_expr(expr.right)
                if isinstance(left, (int, float)) and isinstance(right, (int, float)):
                    try:
                        if expr.op == '+':
                            result = left + right
                        elif expr.op == '-':
                            result = left - right
                        elif expr.op == '*':
                            result = left * right
                        elif expr.op == '/' and right != 0:
                            result = left / right
                        else:
                            return expr  # Cannot fold
                        self.changed = True
                        return result
                    except:
                        return expr  # Preserve on error
                expr.left = left
                expr.right = right
            elif hasattr(expr, 'value'):
                return expr.value
            return expr

        # Traverse and fold (placeholder traversal)
        self.changed = True  # Assume change for iteration

    def dead_code_elimination(self):
        """Remove unreachable code safely"""
        # Check for unreachable blocks after returns
        # Placeholder: mark as changed if any removal
        pass

    def inline_functions(self):
        """Inline small functions without side effects"""
        # Only inline functions < 5 statements without external calls
        # Placeholder: ensure no semantic change
        pass