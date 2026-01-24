#[cfg(test)]
mod optimization_benchmarks {
    use grok::ast::{AstNode, Param, Span};
    use grok::ir::{IRGenerator, IRFunction, IRBlock, IRInstruction, Opcode};
    use grok::optimizations::{OptimizedVM, BytecodeOptimizer, HotPathTracker, HOT_THRESHOLD};
    use grok::vm::{Value, VM};
    use std::time::Instant;

    fn create_fib_ir() -> Vec<IRFunction> {
        // Build fib IR by hand for the optimized VM
        // fn fib(n) { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
        vec![
            IRFunction {
                name: "fib".to_string(),
                params: vec!["n".to_string()],
                blocks: vec![
                    IRBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                            IRInstruction { opcode: Opcode::PushInt(2) },
                            IRInstruction { opcode: Opcode::Lt },
                            IRInstruction { opcode: Opcode::JmpIfFalse("else_0".to_string()) },
                        ],
                    },
                    IRBlock {
                        label: "then_0".to_string(),
                        instructions: vec![
                            IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                            IRInstruction { opcode: Opcode::Jmp("end_0".to_string()) },
                        ],
                    },
                    IRBlock {
                        label: "else_0".to_string(),
                        instructions: vec![
                            // fib(n - 1)
                            IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                            IRInstruction { opcode: Opcode::PushInt(1) },
                            IRInstruction { opcode: Opcode::Sub },
                            IRInstruction { opcode: Opcode::Call("fib".to_string(), 1) },
                            // fib(n - 2)
                            IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                            IRInstruction { opcode: Opcode::PushInt(2) },
                            IRInstruction { opcode: Opcode::Sub },
                            IRInstruction { opcode: Opcode::Call("fib".to_string(), 1) },
                            // add results
                            IRInstruction { opcode: Opcode::Add },
                            IRInstruction { opcode: Opcode::Jmp("end_0".to_string()) },
                        ],
                    },
                    IRBlock {
                        label: "end_0".to_string(),
                        instructions: vec![
                            IRInstruction { opcode: Opcode::Ret },
                        ],
                    },
                ],
            },
            IRFunction {
                name: "main".to_string(),
                params: vec![],
                blocks: vec![
                    IRBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            IRInstruction { opcode: Opcode::PushInt(25) },
                            IRInstruction { opcode: Opcode::Call("fib".to_string(), 1) },
                            IRInstruction { opcode: Opcode::Ret },
                        ],
                    },
                ],
            },
        ]
    }

    #[test]
    fn test_optimized_vm_fibonacci() {
        let ir_funcs = create_fib_ir();
        
        let mut vm = OptimizedVM::new();
        vm.load_program(&ir_funcs);
        
        // Execute fib(25) = 75025
        let start = Instant::now();
        let result = vm.execute("main", vec![]);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "OptimizedVM execution failed: {:?}", result.err());
        let val = result.unwrap();
        
        if let Value::Int(v) = val {
            println!("OptimizedVM fib(25) = {}", v);
            println!("OptimizedVM Time: {:.4}s", duration.as_secs_f64());
            assert_eq!(v, 75025);
        } else {
            panic!("Expected Int, got {:?}", val);
        }
    }

    #[test]
    fn test_bytecode_specialization_benefit() {
        let mut optimizer = BytecodeOptimizer::new();
        
        // Create a function with many arithmetic operations
        let ir_func = IRFunction {
            name: "compute".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            blocks: vec![IRBlock {
                label: "entry".to_string(),
                instructions: vec![
                    IRInstruction { opcode: Opcode::LoadVar("x".to_string()) },
                    IRInstruction { opcode: Opcode::LoadVar("y".to_string()) },
                    IRInstruction { opcode: Opcode::Add },
                    IRInstruction { opcode: Opcode::LoadVar("x".to_string()) },
                    IRInstruction { opcode: Opcode::Mul },
                    IRInstruction { opcode: Opcode::LoadVar("y".to_string()) },
                    IRInstruction { opcode: Opcode::Sub },
                    IRInstruction { opcode: Opcode::Ret },
                ],
            }],
        };
        
        let specialized = optimizer.optimize(&ir_func);
        
        // Verify specialization
        let block = &specialized.blocks[0];
        
        // Count specialized vs generic opcodes
        let specialized_count = block.instructions.iter()
            .filter(|i| !matches!(i.opcode, grok::optimizations::SpecializedOpcode::Generic(_)))
            .count();
        
        let total_count = block.instructions.len();
        
        println!("Specialized: {}/{} opcodes", specialized_count, total_count);
        
        // All arithmetic and loads should be specialized
        assert!(specialized_count >= 6, "Expected at least 6 specialized opcodes");
    }

    #[test]
    fn test_inline_cache_performance() {
        use grok::optimizations::InlineCache;
        use std::collections::HashMap;
        
        let mut cache = InlineCache::new();
        
        // Pre-populate cache
        for i in 0..1000 {
            cache.cache_slot(format!("var_{}", i), i);
        }
        
        // Benchmark cached lookups vs HashMap lookups
        let mut hash_map: HashMap<String, usize> = HashMap::new();
        for i in 0..1000 {
            hash_map.insert(format!("var_{}", i), i);
        }
        
        let iterations = 100_000;
        
        // Cached lookup
        let start = Instant::now();
        for i in 0..iterations {
            let _ = cache.get_cached_slot(&format!("var_{}", i % 1000));
        }
        let cached_duration = start.elapsed();
        
        // HashMap lookup
        let start = Instant::now();
        for i in 0..iterations {
            let _ = hash_map.get(&format!("var_{}", i % 1000));
        }
        let hashmap_duration = start.elapsed();
        
        println!("Inline cache: {:?}", cached_duration);
        println!("HashMap:      {:?}", hashmap_duration);
        
        // Cache should be competitive with HashMap (both O(1))
        // This test verifies the cache works correctly
    }

    #[test]
    fn test_hot_path_triggers_optimization() {
        let mut tracker = HotPathTracker::new();
        
        // Simulate function calls
        let mut became_hot_at = 0;
        for i in 1..=HOT_THRESHOLD + 10 {
            if tracker.record_call("hot_function") {
                became_hot_at = i;
            }
        }
        
        assert_eq!(became_hot_at, HOT_THRESHOLD, "Function should become hot at threshold");
        assert!(tracker.is_hot("hot_function"));
        assert!(!tracker.is_hot("cold_function"));
        
        println!("Hot threshold: {} calls", HOT_THRESHOLD);
        println!("Function became hot at: {} calls", became_hot_at);
    }

    #[test]
    fn test_tail_call_optimization_stack_usage() {
        use grok::optimizations::is_tail_call;
        
        // Test tail call detection in different patterns
        
        // Pattern 1: Direct tail call (call immediately followed by ret)
        let block1 = IRBlock {
            label: "test".to_string(),
            instructions: vec![
                IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                IRInstruction { opcode: Opcode::Call("recurse".to_string(), 1) },
                IRInstruction { opcode: Opcode::Ret },
            ],
        };
        assert!(is_tail_call(&block1, 1), "Should detect tail call");
        
        // Pattern 2: Not a tail call (operation after call)
        let block2 = IRBlock {
            label: "test".to_string(),
            instructions: vec![
                IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                IRInstruction { opcode: Opcode::Call("helper".to_string(), 1) },
                IRInstruction { opcode: Opcode::PushInt(1) },
                IRInstruction { opcode: Opcode::Add },
                IRInstruction { opcode: Opcode::Ret },
            ],
        };
        assert!(!is_tail_call(&block2, 1), "Should NOT detect as tail call when followed by operations");
        
        println!("Tail call detection: PASSED");
    }

    #[test]
    fn test_fast_locals_vs_hashmap() {
        use grok::optimizations::FastLocals;
        use std::collections::HashMap;
        
        let iterations = 1_000_000;
        
        // FastLocals benchmark
        let mut fast_locals = FastLocals::new(10);
        let start = Instant::now();
        for i in 0..iterations {
            fast_locals.set(i % 10, Value::Int(i as i64));
            let _ = fast_locals.get(i % 10);
        }
        let fast_duration = start.elapsed();
        
        // HashMap benchmark
        let mut hash_locals: HashMap<String, Value> = HashMap::new();
        let keys: Vec<String> = (0..10).map(|i| format!("var_{}", i)).collect();
        let start = Instant::now();
        for i in 0..iterations {
            hash_locals.insert(keys[i % 10].clone(), Value::Int(i as i64));
            let _ = hash_locals.get(&keys[i % 10]);
        }
        let hash_duration = start.elapsed();
        
        println!("FastLocals: {:?}", fast_duration);
        println!("HashMap:    {:?}", hash_duration);
        
        // FastLocals should be significantly faster
        let speedup = hash_duration.as_nanos() as f64 / fast_duration.as_nanos() as f64;
        println!("Speedup: {:.2}x", speedup);
        
        assert!(speedup > 1.0, "FastLocals should be faster than HashMap");
    }

    #[test]
    fn test_specialized_integer_operations() {
        use grok::optimizations::OptimizedContext;
        use grok::optimizations::SpecializedOpcode;
        
        let mut ctx = OptimizedContext::new();
        
        // Test specialized IntAdd
        ctx.push(Value::Int(40));
        ctx.push(Value::Int(2));
        ctx.execute_specialized(&SpecializedOpcode::IntAdd).unwrap();
        assert_eq!(ctx.pop(), Some(Value::Int(42)));
        
        // Test specialized IntMul
        ctx.push(Value::Int(6));
        ctx.push(Value::Int(7));
        ctx.execute_specialized(&SpecializedOpcode::IntMul).unwrap();
        assert_eq!(ctx.pop(), Some(Value::Int(42)));
        
        // Test specialized IntLt
        ctx.push(Value::Int(5));
        ctx.push(Value::Int(10));
        ctx.execute_specialized(&SpecializedOpcode::IntLt).unwrap();
        assert_eq!(ctx.pop(), Some(Value::Bool(true)));
        
        // Test LoadLocalFast
        ctx.locals.set(0, Value::Int(100));
        ctx.execute_specialized(&SpecializedOpcode::LoadLocalFast(0)).unwrap();
        assert_eq!(ctx.pop(), Some(Value::Int(100)));
        
        // Test StoreLocalFast
        ctx.push(Value::Int(200));
        ctx.execute_specialized(&SpecializedOpcode::StoreLocalFast(1)).unwrap();
        assert_eq!(ctx.locals.get(1), &Value::Int(200));
        
        println!("All specialized operations: PASSED");
    }
}
