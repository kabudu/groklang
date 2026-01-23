# GrokLang FFI example
import sys
import os

# We need to point to the built shared library
# On Mac/Linux it effectively is .so or .dylib, but Python looks for .so or .pyd usually
# Cargo builds libgrok.dylib on Mac. We might need to rename or symlink it.

# Assuming we are running from project root
# For testing locally without full install:
try:
    import grok
except ImportError:
    # Try finding it in target/debug
    import sys
    sys.path.append('target/debug')
    # On mac, cargo produces libgrok.dylib. Python expects grok.so
    # We will rename it in the test script
    try:
        import grok
    except ImportError:
        print('Could not find grok module. Make sure it is built and renamed.')
        sys.exit(1)

g = grok.GrokInterpreter()
code = 'fn main() -> i64 { return 42 + 10; }'
print(f'Running code: {code}')
result = g.run(code)
print(f'Result from Rust: {result}')

