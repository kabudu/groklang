# tests/test_error_diagnostics.py
"""Tests for Enhanced Error Diagnostics"""

import pytest
from groklang.type_checker import TypeChecker

class TestErrorDiagnostics:
    def test_type_checker_errors(self):
        checker = TypeChecker()
        # Invalid AST
        ast = ("Program", [])
        substitutions = checker.check(ast)
        # Should not crash, but may have errors
        assert isinstance(checker.errors, list)

    def test_add_error(self):
        checker = TypeChecker()
        checker.add_error("Test error", "Fix it")
        assert len(checker.errors) == 1
        assert "Suggestion: Fix it" in checker.errors[0]