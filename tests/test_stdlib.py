# tests/test_stdlib.py
"""Tests for GrokLang Standard Library"""

import pytest
from groklang.stdlib import Vec, HashMap, Future

class TestStdlib:
    def test_vec(self):
        v = Vec()
        v.push(1)
        v.push(2)
        assert v.len() == 2
        assert v.pop() == 2
        assert v.len() == 1

    def test_hashmap(self):
        hm = HashMap()
        hm.insert('key', 'value')
        assert hm.get('key') == 'value'
        assert hm.contains('key')
        assert not hm.contains('missing')

    def test_future(self):
        def func():
            return 42
        f = Future(func)
        assert f.await_result() == 42
        assert f.done