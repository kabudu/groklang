#!/usr/bin/env python3
"""GrokLang Compiler Entry Point"""

import sys
import argparse
from groklang.compiler import Compiler

def main():
    parser = argparse.ArgumentParser(description="GrokLang Compiler")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Compile command (default)
    compile_parser = subparsers.add_parser("compile", help="Compile GrokLang source")
    compile_parser.add_argument("file", help="GrokLang source file (.grok)")
    compile_parser.add_argument("--target", choices=["vm", "llvm"], default="vm",
                                help="Compilation target (default: vm)")
    compile_parser.add_argument("--run", action="store_true",
                                help="Execute the program after compilation")

    # LSP command
    lsp_parser = subparsers.add_parser("lsp", help="Start Language Server Protocol server")

    # Debug command
    debug_parser = subparsers.add_parser("debug", help="Debug GrokLang source")
    debug_parser.add_argument("file", help="GrokLang source file (.grok)")

    # Package commands
    new_parser = subparsers.add_parser("new", help="Create a new GrokLang project")
    new_parser.add_argument("name", help="Project name")

    install_parser = subparsers.add_parser("install", help="Install project dependencies")

    build_parser = subparsers.add_parser("build", help="Build the project")

    args = parser.parse_args()

    if args.command == "lsp":
        from groklang.language_server import server
        import asyncio
        asyncio.run(server.start_io())
        return

    if args.command == "debug":
        from groklang.debugger import debug_file
        debug_file(args.file)
        return

    if args.command == "new":
        from groklang.package_manager import new_project
        new_project(args.name)
        return

    if args.command == "install":
        from groklang.package_manager import install_deps
        install_deps()
        return

    if args.command == "build":
        from groklang.package_manager import build_project
        build_project()
        return

    # For backward compatibility, assume compile if no command
    if args.command is None:
        # Re-parse as compile
        compile_parser.parse_args(sys.argv[1:], namespace=args)
        args.command = "compile"

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
        result = gen.compile_to_executable(args.file)
        print(result)
    elif args.target == "vm" and args.run:
        print("Program compiled successfully")

if __name__ == "__main__":
    main()