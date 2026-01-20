# groklang/debugger.py
"""GrokLang Debugger for VM execution"""

import sys
from groklang.vm import BytecodeVM as VM

class Debugger:
    def __init__(self, code, ast):
        self.vm = VM()
        self.breakpoints = set()
        self.current_line = 0
        self.running = False
        self.step_mode = False

        # Compile to bytecode
        from groklang.compiler import Compiler
        compiler = Compiler()
        result = compiler.compile(code, target="vm")
        self.bytecode = result['bytecode']

    def add_breakpoint(self, line):
        self.breakpoints.add(line)

    def remove_breakpoint(self, line):
        self.breakpoints.discard(line)

    def run(self):
        self.running = True
        print("Starting debugger...")
        while self.running and self.vm.pc < len(self.bytecode):
            if self.step_mode or self.current_line in self.breakpoints:
                self.pause()
            self.vm.step()
            # Update current_line from bytecode (simplified)
            self.current_line += 1

    def pause(self):
        print(f"Paused at line {self.current_line}")
        while True:
            cmd = input("(grok-debug) ").strip().split()
            if not cmd:
                continue
            command = cmd[0]
            args = cmd[1:]

            if command == "continue" or command == "c":
                break
            elif command == "step" or command == "s":
                self.step_mode = True
                break
            elif command == "break" or command == "b":
                if args:
                    try:
                        line = int(args[0])
                        self.add_breakpoint(line)
                        print(f"Breakpoint set at line {line}")
                    except ValueError:
                        print("Invalid line number")
                else:
                    print(f"Breakpoints: {sorted(self.breakpoints)}")
            elif command == "clear":
                if args:
                    try:
                        line = int(args[0])
                        self.remove_breakpoint(line)
                        print(f"Cleared breakpoint at line {line}")
                    except ValueError:
                        print("Invalid line number")
                else:
                    self.breakpoints.clear()
                    print("All breakpoints cleared")
            elif command == "inspect" or command == "i":
                if args:
                    var = args[0]
                    # Simplified: assume vars in vm.stack
                    print(f"Inspect {var}: Not implemented yet")
                else:
                    print(f"Stack: {self.vm.stack}")
            elif command == "quit" or command == "q":
                self.running = False
                break
            elif command == "help" or command == "h":
                print("""Commands:
  continue (c)       - Continue execution
  step (s)           - Step to next instruction
  break (b) [line]   - Set breakpoint or list breakpoints
  clear [line]       - Clear breakpoint(s)
  inspect (i) [var]  - Inspect variable or stack
  quit (q)           - Exit debugger
  help (h)           - Show this help
""")
            else:
                print(f"Unknown command: {command}")

    def step(self):
        self.step_mode = True
        self.run()

def debug_file(filename):
    try:
        with open(filename, 'r') as f:
            code = f.read()
    except FileNotFoundError:
        print(f"Error: File '{filename}' not found")
        return

    from groklang import parser as parser_mod
    ast = parser_mod.parser.parse(code)
    if parser_mod.parser.errors:
        print("Parse errors:")
        for error in parser_mod.parser.errors:
            print(f"  {error}")
        return

    debugger = Debugger(code, ast)
    debugger.run()