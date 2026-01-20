# tests/test_debugger.py
"""Tests for GrokLang Debugger"""

import pytest
from unittest.mock import Mock, patch
from groklang.debugger import Debugger

class TestDebugger:
    def test_debugger_init(self):
        code = "fn main() { let x = 42; }"
        ast = Mock()
        debugger = Debugger(code, ast)
        assert isinstance(debugger.vm, Mock)  # Simplified, VM is mocked

    def test_add_breakpoint(self):
        code = "fn main() {}"
        ast = Mock()
        debugger = Debugger(code, ast)
        debugger.add_breakpoint(5)
        assert 5 in debugger.breakpoints

    def test_remove_breakpoint(self):
        code = "fn main() {}"
        ast = Mock()
        debugger = Debugger(code, ast)
        debugger.add_breakpoint(5)
        debugger.remove_breakpoint(5)
        assert 5 not in debugger.breakpoints

    @patch('builtins.input', side_effect=['continue'])
    def test_pause_continue(self, mock_input):
        code = "fn main() {}"
        ast = Mock()
        debugger = Debugger(code, ast)
        debugger.current_line = 10
        debugger.breakpoints.add(10)
        with patch.object(debugger.vm, 'step'):
            debugger.pause()
            assert not debugger.step_mode

    @patch('builtins.input', side_effect=['step'])
    def test_pause_step(self, mock_input):
        code = "fn main() {}"
        ast = Mock()
        debugger = Debugger(code, ast)
        debugger.current_line = 10
        debugger.breakpoints.add(10)
        with patch.object(debugger.vm, 'step'):
            debugger.pause()
            assert debugger.step_mode