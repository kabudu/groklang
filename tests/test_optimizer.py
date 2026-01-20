# tests/test_optimizer.py
"""Tests for GrokLang Optimizer"""

import pytest
from groklang.optimizer import Optimizer

class TestOptimizer:
    def test_optimizer_init(self):
        ast = ("Program", [])
        opt = Optimizer(ast)
        assert opt.ast == ast

    def test_optimize(self):
        ast = ("Program", [])
        opt = Optimizer(ast)
        result = opt.optimize()
        assert result == ast  # Placeholder, no changes yet