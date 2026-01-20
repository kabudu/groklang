# groklang/stdlib.py
"""GrokLang Standard Library"""

class Vec:
    """Dynamic array"""
    def __init__(self):
        self.data = []

    def push(self, item):
        self.data.append(item)

    def pop(self):
        return self.data.pop()

    def len(self):
        return len(self.data)

class HashMap:
    """Hash map"""
    def __init__(self):
        self.data = {}

    def insert(self, key, value):
        self.data[key] = value

    def get(self, key):
        return self.data.get(key)

    def contains(self, key):
        return key in self.data

# I/O functions
def println(msg):
    print(msg)

def readln():
    return input()

# Async primitives (simplified)
class Future:
    def __init__(self, func):
        self.func = func
        self.result = None
        self.done = False

    def await_result(self):
        if not self.done:
            self.result = self.func()
            self.done = True
        return self.result

def spawn(func):
    # Simplified: just call func
    return Future(func)