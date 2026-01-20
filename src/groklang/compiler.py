from . import parser
from .type_checker import TypeChecker
from .decorator_processor import DecoratorProcessor, MockLlmService, OpenAiService, XaiGrokService
from .codegen import CodeGenerator
from .vm import BytecodeVM
from .llvm_codegen import LLVMGenerator
from .config import config
from .deadlock_detector import DeadlockDetector
from .runtime_ai import RuntimeAIOptimizer
from .macro_expander import MacroExpander
from .module_system import ModuleResolver, PrivacyChecker
from .jit_compiler import JITCompiler
from .advanced_gc import AdvancedGC
from .zero_cost_optimizer import ZeroCostOptimizer
from .security import SandboxRunner, FormalVerifier
from .deadlock_detector import DeadlockDetector
from .optimizer import Optimizer

class Compiler:
    def __init__(self):
        self.type_checker = TypeChecker()
        self.llm_service = self.create_llm_service()
        self.decorator_processor = DecoratorProcessor(self.llm_service)
        self.deadlock_detector = DeadlockDetector(self.llm_service) if config.deadlock_detection else None
        self.runtime_ai = RuntimeAIOptimizer(self.llm_service, BytecodeVM().profiler)
        self.macro_expander = MacroExpander()
        self.module_resolver = ModuleResolver()
        self.privacy_checker = PrivacyChecker(self.module_resolver)
        self.jit_compiler = JITCompiler()
        self.gc = AdvancedGC()
        self.zero_cost_optimizer = ZeroCostOptimizer(self.llm_service)
        self.optimizer = Optimizer(None)  # Will set ast later
        self.codegen = CodeGenerator()
        self.vm = BytecodeVM()
        self.llvm_gen = LLVMGenerator()

    def create_llm_service(self):
        backend = config.ai_backend
        if backend == "openai":
            if not config.ai_api_key:
                raise ValueError("OpenAI API key not configured")
            return OpenAiService(config.ai_api_key)
        elif backend == "xai":
            if not config.ai_api_key:
                raise ValueError("XAI API key not configured")
            return XaiGrokService(config.ai_api_key)
        else:
            return MockLlmService()

    def compile(self, code: str, target='vm', optimize=True):
        # Parse
        ast = parser.parser.parse(code)
        if parser.parser.errors:
            raise SyntaxError(f"Parse errors: {parser.parser.errors}")
        if ast is None or not isinstance(ast, tuple) or len(ast) < 2:
            raise SyntaxError("Failed to parse code")

        # Expand macros
        ast = self.macro_expander.expand_ast(ast)

        # Process modules and privacy
        if isinstance(ast, tuple) and len(ast) > 1:
            for item in ast[1]:
                if isinstance(item, tuple) and item[0] == 'Module':
                    self.module_resolver.define_module(item[1], item[2])
        
        # Check privacy
        privacy_errors = self.privacy_checker.check_privacy(ast)
        if privacy_errors:
            raise SyntaxError(f"Privacy errors: {privacy_errors}")

        # Process decorators
        ast = self.decorator_processor.process_decorators(ast)

        # Zero-cost optimizations
        code = self.zero_cost_optimizer.optimize_abstractions(code)

        # Deadlock detection
        if self.deadlock_detector:
            deadlock_analysis = self.deadlock_detector.analyze_code(code)
            if deadlock_analysis['risk_level'] == 'high':
                print("Warning: High deadlock risk detected!")
                print(f"Recommendations: {deadlock_analysis['recommendations']}")

        # Type check
        substitutions = self.type_checker.check(ast)
        errors = self.type_checker.errors

        # Optimize AST
        if optimize:
            self.optimizer.ast = ast
            ast = self.optimizer.optimize()

        # Generate code
        if target == 'vm':
            ir_functions = self.codegen.generate(ast)
            self.vm.load_program(ir_functions)
            return {
                'ast': ast,
                'ir': ir_functions,
                'vm': self.vm,
                'errors': errors
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
        elif target == 'jit':
            ir_functions = self.codegen.generate(ast)
            llvm_code = self.llvm_gen.generate(ir_functions)
            jit_result = self.jit_compiler.compile_and_run(llvm_code)
            return {
                'ast': ast,
                'ir': ir_functions,
                'llvm': llvm_code,
                'jit_result': jit_result,
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

    def runtime_optimize(self, code: str) -> str:
        """Apply runtime AI optimizations"""
        return self.runtime_ai.optimize_hotspots(code)