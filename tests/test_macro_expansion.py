# tests/test_macro_expansion.py
"""Tests for Macro Expansion Correctness"""

import pytest
from groklang.macro_expander import MacroExpander

def test_macro_definition():
    expander = MacroExpander()
    rules = [(["$x"], "($x + 1)")]
    expander.define_macro("inc", rules)
    assert "inc" in expander.macros

def test_macro_expansion():
    expander = MacroExpander()
    rules = [(["$x"], "($x + 1)")]
    expander.define_macro("inc", rules)
    result = expander.expand_macro("inc", [5])
    assert result == "(5 + 1)"

def test_macro_no_match():
    expander = MacroExpander()
    rules = [(["$x"], "($x + 1)")]
    expander.define_macro("inc", rules)
    with pytest.raises(ValueError):
        expander.expand_macro("inc", [5, 6])  # Wrong args