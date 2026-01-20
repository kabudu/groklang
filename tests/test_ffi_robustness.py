# tests/test_ffi_robustness.py
"""Tests for FFI Robustness"""

import pytest
from groklang.ffi import TypeMarshaler
from groklang.types import PrimitiveType

def test_grok_to_c_validation():
    marshaler = TypeMarshaler()
    type_i32 = PrimitiveType("i32")
    
    # Valid
    assert marshaler.grok_to_c(42, type_i32) == 42
    
    # Invalid type
    with pytest.raises(TypeError):
        marshaler.grok_to_c("not int", type_i32)
    
    # Out of range
    with pytest.raises(ValueError):
        marshaler.grok_to_c(2**32, type_i32)

def test_c_to_grok_validation():
    marshaler = TypeMarshaler()
    type_i32 = PrimitiveType("i32")
    
    # Valid
    assert marshaler.c_to_grok(42, type_i32) == 42
    
    # Out of range
    with pytest.raises(ValueError):
        marshaler.c_to_grok(2**32, type_i32)