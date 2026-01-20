#!/usr/bin/env python3
"""Fuzz testing script for GrokLang parser and compiler"""

import random
import string
from groklang import parser as parser_mod
from groklang.compiler import Compiler

def generate_random_code(max_length=1000):
    """Generate random code snippets for fuzzing"""
    keywords = ['fn', 'let', 'if', 'else', 'for', 'while', 'return', 'struct', 'enum']
    ops = ['+', '-', '*', '/', '=', '==', '!=', '<', '>', '&&', '||']
    tokens = keywords + ops + [str(random.randint(0, 100)) for _ in range(10)] + \
             [f'id{random.randint(0, 10)}' for _ in range(10)] + \
             ['(', ')', '{', '}', '[', ']', ';', ',']

    code = []
    length = random.randint(10, max_length)
    for _ in range(length):
        code.append(random.choice(tokens))
    return ' '.join(code)

def fuzz_test(iterations=100):
    """Run fuzz tests"""
    compiler = Compiler()
    crashes = 0
    errors = 0

    for i in range(iterations):
        try:
            code = generate_random_code()
            ast = parser_mod.parser.parse(code)
            if ast:
                result = compiler.compile(code)
            print(f"Iteration {i+1}: OK")
        except RecursionError:
            print(f"Iteration {i+1}: Recursion limit hit")
            crashes += 1
        except Exception as e:
            print(f"Iteration {i+1}: Error - {e}")
            errors += 1

    print(f"\nFuzz test complete: {crashes} crashes, {errors} errors out of {iterations}")

if __name__ == "__main__":
    fuzz_test()