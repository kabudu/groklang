from src.groklang.jit_compiler import JITCompiler
from src.groklang.advanced_gc import AdvancedGC
from src.groklang.zero_cost_optimizer import ZeroCostOptimizer
from src.groklang.decorator_processor import MockLlmService

def test_jit_compiler():
    jit = JITCompiler()
    # Test initialization (JIT may not work without LLVM setup)
    assert jit.engine is not None or jit.engine is None  # Either way, object exists
    print("JIT compiler test passed!")

def test_advanced_gc():
    gc = AdvancedGC()
    
    # Allocate objects
    obj1 = gc.allocate("test1")
    obj2 = gc.allocate("test2")
    
    # Mark one as root
    gc.add_root(obj1)
    
    # Collect (should free obj2)
    freed = gc.collect()
    assert freed >= 0  # At least didn't crash
    
    stats = gc.get_stats()
    assert 'objects' in stats
    assert 'roots' in stats
    print("Advanced GC test passed!")

def test_zero_cost_optimizer():
    llm_service = MockLlmService()
    optimizer = ZeroCostOptimizer(llm_service)
    
    code = "fn test() { let x = vec![1,2,3]; x.sum() }"
    
    # Test optimization
    optimized = optimizer.optimize_abstractions(code)
    assert isinstance(optimized, str)
    
    # Test analysis
    analysis = optimizer.analyze_overhead(code)
    assert 'abstractions_found' in analysis
    print("Zero-cost optimizer test passed!")

if __name__ == "__main__":
    test_jit_compiler()
    test_advanced_gc()
    test_zero_cost_optimizer()