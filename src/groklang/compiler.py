from . import parser
from .type_checker import TypeChecker
from .decorator_processor import DecoratorProcessor, MockLlmService
from .codegen import CodeGenerator
from .vm import BytecodeVM
from .llvm_codegen import LLVMGenerator

class Compiler:
    def __init__(self):
        self.type_checker = TypeChecker()
        self.decorator_processor = DecoratorProcessor(MockLlmService())
        self.codegen = CodeGenerator()
        self.vm = BytecodeVM()
        self.llvm_gen = LLVMGenerator()

    def compile(self, code: str, target='vm'):
        # Parse
        ast = parser.parser.parse(code)
        if parser.parser.errors:
            raise SyntaxError(f"Parse errors: {parser.parser.errors}")

        # Process decorators
        ast = self.decorator_processor.process_decorators(ast)

        # Type check
        substitutions = self.type_checker.check(ast)

        # Generate code
        if target == 'vm':
            ir_functions = self.codegen.generate(ast)
            self.vm.load_program(ir_functions)
            return {
                'ast': ast,
                'ir': ir_functions,
                'vm': self.vm,
                'errors': []
            }
        elif target == 'llvm':
            ir_functions = self.codegen.generate(ast)
            llvm_code = self.llvm_gen.generate(ir_functions)
            return {
                'ast': ast,
                'ir': ir_functions,
                'llvm': llvm_code,
                'errors': []
            }
        else:
            return {
                'ast': ast,
                'substitutions': substitutions,
                'errors': []
            }