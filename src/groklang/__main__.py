#!/usr/bin/env python3
"""GrokLang Compiler Entry Point"""

import sys
import argparse
from groklang.compiler import Compiler

def main():
    parser = argparse.ArgumentParser(description="GrokLang Compiler")
    parser.add_argument("file", help="GrokLang source file (.grok)")
    parser.add_argument("--target", choices=["vm", "llvm"], default="vm",
                        help="Compilation target (default: vm)")
    parser.add_argument("--run", action="store_true",
                        help="Execute the program after compilation")

    args = parser.parse_args()

    # Read source file
    try:
        with open(args.file, 'r') as f:
            code = f.read()
    except FileNotFoundError:
        print(f"Error: File '{args.file}' not found")
        sys.exit(1)

    # Parse first to check for errors
    from groklang import parser as parser_mod
    ast = parser_mod.parser.parse(code)
    print(f"Parsed AST: {ast}")  # Debug
    if parser_mod.parser.errors:
        print("Parse errors:")
        for error in parser_mod.parser.errors:
            print(f"  {error}")
        sys.exit(1)
    if ast is None:
        print("Parse returned None")
        sys.exit(1)

    # Compile
    compiler = Compiler()
    try:
        result = compiler.compile(code, target=args.target)
        if 'errors' in result and result['errors']:
            print("Compilation errors:")
            for error in result['errors']:
                print(f"  {error}")
            sys.exit(1)
    except Exception as e:
        print(f"Compilation failed: {e}")
        sys.exit(1)

    print(f"Successfully compiled '{args.file}'")

    if args.target == "llvm":
        llvm_file = args.file.replace('.grok', '.ll')
        with open(llvm_file, 'w') as f:
            f.write(result['llvm'])
        print(f"LLVM IR written to '{llvm_file}'")

    elif args.target == "llvm" and args.run:
        from groklang.llvm_codegen import LLVMGenerator
        gen = LLVMGenerator()
        result = gen.save_and_compile(args.file)
        print(result)
    elif args.target == "vm" and args.run:
        print("Program compiled successfully")

if __name__ == "__main__":
    main()