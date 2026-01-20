# tests/test_optimization_equivalence.py
"""Tests for optimization equivalence"""

import pytest
from groklang.compiler import Compiler

def test_optimization_equivalence():
    code = '''
fn main() {
    let x = 1 + 2 * 3;
    println(x);
}
'''
    compiler = Compiler()
    
    # Compile with optimization
    result_opt = compiler.compile(code, target='vm', optimize=True)
    assert not result_opt['errors']
    vm_opt = result_opt['vm']
    
    # Compile without optimization
    result_no_opt = compiler.compile(code, target='vm', optimize=False)
    assert not result_no_opt['errors']
    vm_no_opt = result_no_opt['vm']
    
    # For now, just check they compile. Full equivalence needs execution comparison.
    # TODO: Run both VMs and compare outputs
    assert vm_opt is not None
    assert vm_no_opt is not None