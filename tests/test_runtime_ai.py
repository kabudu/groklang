from src.groklang.runtime_ai import RuntimeAIOptimizer, RuntimeProfiler
from src.groklang.decorator_processor import MockLlmService

def test_runtime_profiler():
    profiler = RuntimeProfiler()
    
    # Record executions
    profiler.record_execution("func1")
    profiler.record_execution("func1")
    profiler.record_execution("func2")
    
    for _ in range(105):  # Exceed threshold
        profiler.record_execution("hot_func")
    
    hotspots = profiler.get_hotspots()
    assert "hot_func" in hotspots
    assert "func1" not in hotspots
    assert "func2" not in hotspots
    print("Runtime profiler test passed!")

def test_runtime_ai_optimizer():
    llm_service = MockLlmService()
    profiler = RuntimeProfiler()
    optimizer = RuntimeAIOptimizer(llm_service, profiler)
    
    # No hotspots initially
    code = "fn test() { }"
    optimized = optimizer.optimize_hotspots(code)
    assert optimized == code  # No change
    
    # Add hotspot
    for _ in range(105):
        profiler.record_execution("hot_func")
    
    optimized = optimizer.optimize_hotspots(code)
    # Mock service returns the input, so should be same
    assert optimized == code
    print("Runtime AI optimizer test passed!")

if __name__ == "__main__":
    test_runtime_profiler()
    test_runtime_ai_optimizer()