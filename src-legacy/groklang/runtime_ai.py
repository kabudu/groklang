from .decorator_processor import LlmService
from .vm import RuntimeProfiler

class RuntimeAIOptimizer:
    def __init__(self, llm_service: LlmService, profiler: RuntimeProfiler):
        self.llm_service = llm_service
        self.profiler = profiler

    def optimize_hotspots(self, code: str) -> str:
        """Use AI to optimize hot functions at runtime"""
        hotspots = self.profiler.get_hotspots()
        if not hotspots:
            return code

        # Get profiling data
        profile_data = f"Hot functions: {hotspots}\nExecution counts: {self.profiler.execution_counts}"

        # Ask AI for optimizations
        request = {
            'operation': 'runtime_optimize',
            'input': code,
            'parameters': {'profile': profile_data, 'hotspots': hotspots}
        }

        response = self.llm_service.call(request)
        if response['success']:
            return response['output']  # Optimized code
        return code  # Fallback

    def should_recompile(self) -> bool:
        """Check if recompilation is needed"""
        return len(self.profiler.get_hotspots()) > 0