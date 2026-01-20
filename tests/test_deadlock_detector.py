# tests/test_deadlock_detector.py
"""Tests for Deadlock Detector Edge Cases"""

import pytest
from groklang.deadlock_detector import DeadlockDetector
from unittest.mock import Mock

def test_static_deadlock_analysis():
    llm = Mock()
    detector = DeadlockDetector(llm)
    
    # Code with nested locks
    code = "lock1.lock(); lock2.lock();"
    result = detector._static_deadlock_analysis(code)
    assert "nested locking" in result['issues'][0].lower()
    assert result['risk_level'] in ['low', 'medium', 'high']

def test_ai_deadlock_analysis():
    llm = Mock()
    llm.call.return_value = {'risk_level': 'medium', 'recommendations': ['Add timeouts']}
    detector = DeadlockDetector(llm)
    
    result = detector._ai_deadlock_analysis("code")
    assert result['risk_level'] == 'medium'

def test_combined_risks():
    llm = Mock()
    detector = DeadlockDetector(llm)
    
    static = {'risk_level': 'high'}
    ai = {'risk_level': 'low'}
    combined = detector._combine_risks(static, ai)
    assert combined == 'high'