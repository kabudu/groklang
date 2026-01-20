from . import parser
from .type_checker import TypeChecker
from .decorator_processor import DecoratorProcessor, MockLlmService, OpenAiService, XaiGrokService
from .codegen import CodeGenerator
from .vm import BytecodeVM
from .llvm_codegen import LLVMGenerator
from .config import config
from .runtime_ai import RuntimeAIOptimizer
from .deadlock_detector import DeadlockDetector

class Compiler:
    def __init__(self):
        self.type_checker = TypeChecker()
        self.llm_service = self.create_llm_service()
        self.decorator_processor = DecoratorProcessor(self.llm_service)
        self.deadlock_detector = DeadlockDetector(self.llm_service) if config.deadlock_detection else None
        self.runtime_ai = RuntimeAIOptimizer(self.llm_service, BytecodeVM().profiler)
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

    def compile(self, code: str, target='vm'):
        # Parse
        ast = parser.parser.parse(code)
        if parser.parser.errors:
            raise SyntaxError(f"Parse errors: {parser.parser.errors}")
        if ast is None or not isinstance(ast, tuple) or len(ast) < 2:
            raise SyntaxError("Failed to parse code")

        # Process decorators
        ast = self.decorator_processor.process_decorators(ast)

        # Deadlock detection
        if self.deadlock_detector:
            deadlock_analysis = self.deadlock_detector.analyze_code(code)
            if deadlock_analysis['risk_level'] == 'high':
                print("Warning: High deadlock risk detected!")
                print(f"Recommendations: {deadlock_analysis['recommendations']}")

        # Type check
        substitutions = self.type_checker.check(ast)

        # Generate code (disabled for now)
        if target == 'vm':
            return {
                'ast': ast,
                'ir': [],
                'vm': None,
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