#[cfg(test)]
mod ai_demo_tests {
    use grok::ai::{AiConfig, AiOperation, AiService};
    use std::time::Instant;

    /// Comprehensive AI features demonstration
    /// 
    /// This test demonstrates all AI capabilities of GrokLang:
    /// 1. Multiple providers (Mock, OpenAI, DeepSeek)
    /// 2. All AI operations
    /// 3. Caching and performance
    /// 4. Security auditing
    /// 5. Trace logging
    #[tokio::test]
    async fn test_ai_features_comprehensive() {
        println!("\n======================================");
        println!("GrokLang AI Features Demonstration");
        println!("======================================\n");

        // Create service with environment config (or mock if not configured)
        let mut service = AiService::new();
        println!("Provider: {:?}", service.config().provider);
        println!("Model: {}", service.config().model);
        println!();

        // Sample GrokLang code for demonstration
        let sample_code = r#"
fn fibonacci(n) {
    if n < 2 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

actor Calculator {
    receive {
        (compute, n) => {
            let result = fibonacci(n)
            sender ! (result, result)
        }
        _ => ()
    }
}

fn main() {
    let calc = spawn Calculator {}
    calc ! (compute, 10)
}
"#;

        println!("Sample Code:");
        println!("----------------------------------------");
        println!("{}", sample_code);
        println!("----------------------------------------\n");

        // Test all AI operations
        let operations = vec![
            ("Code Optimization", AiOperation::Optimize),
            ("Code Explanation", AiOperation::Explain),
            ("Bug Detection", AiOperation::Debug),
            ("Code Refactoring", AiOperation::Refactor),
            ("Test Generation", AiOperation::GenerateTests),
            ("Security Audit", AiOperation::SecurityAudit),
            ("Documentation", AiOperation::DocumentCode),
        ];

        for (name, op) in operations {
            println!(">>> Running: {} <<<", name);
            let start = Instant::now();
            
            match service.process(op.clone(), sample_code).await {
                Ok(result) => {
                    let elapsed = start.elapsed();
                    println!("✓ Success ({:.2}ms)", elapsed.as_secs_f64() * 1000.0);
                    println!("Output preview: {}...", 
                        result.chars().take(200).collect::<String>().replace('\n', " "));
                    println!();
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                    println!();
                }
            }
        }

        // Test caching
        println!("\n>>> Testing Cache <<<");
        let start = Instant::now();
        let _ = service.process(AiOperation::Explain, sample_code).await;
        let cached_time = start.elapsed();
        println!("Cached call time: {:.3}ms", cached_time.as_secs_f64() * 1000.0);

        // Print trace report
        println!("\n{}", service.get_trace_report());
    }

    #[tokio::test]
    async fn test_ai_security_checks() {
        println!("\n======================================");
        println!("AI Security Audit Demonstration");
        println!("======================================\n");

        let mut service = AiService::with_config(AiConfig::mock());

        // Safe code
        let safe_code = "fn add(a, b) { a + b }";
        let result = service.process(AiOperation::SecurityAudit, safe_code).await;
        assert!(result.is_ok());
        println!("Safe code audit: ✓");

        // Security check on AI output
        let service_check = AiService::with_config(AiConfig::mock());
        
        let test_cases = vec![
            ("Normal code", "let x = 42", true),
            ("Eval injection", "eval('malicious')", false),
            ("System call", "system('rm -rf /')", false),
            ("SQL injection", "DROP TABLE users", false),
            ("Python import", "__import__('os').system('ls')", false),
        ];

        println!("\nSecurity Pattern Detection:");
        for (name, code, should_be_safe) in test_cases {
            let is_safe = service_check.is_safe(code);
            let status = if is_safe == should_be_safe { "✓" } else { "✗" };
            println!("  {} {}: {} (expected: {})", 
                status, name, 
                if is_safe { "safe" } else { "blocked" },
                if should_be_safe { "safe" } else { "blocked" }
            );
            assert_eq!(is_safe, should_be_safe);
        }
    }

    #[tokio::test]
    async fn test_ai_benchmarks() {
        println!("\n======================================");
        println!("AI Performance Benchmarks");
        println!("======================================\n");

        let mut service = AiService::with_config(AiConfig::mock());

        let code_samples = vec![
            ("Small", "fn f(x) { x + 1 }"),
            ("Medium", "fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n-1) } }"),
            ("Large", r#"
                struct Point { x: i32, y: i32 }
                fn distance(p1: Point, p2: Point) -> f64 {
                    let dx = p2.x - p1.x
                    let dy = p2.y - p1.y
                    sqrt(dx * dx + dy * dy)
                }
                fn main() {
                    let a = Point { x: 0, y: 0 }
                    let b = Point { x: 3, y: 4 }
                    let d = distance(a, b)
                    print(d)
                }
            "#),
        ];

        println!("Operation Latency by Code Size:\n");
        
        for (size_name, code) in &code_samples {
            let iterations = 10;
            let mut total_time = std::time::Duration::ZERO;
            
            for _ in 0..iterations {
                service.clear_cache();
                let start = Instant::now();
                let _ = service.process(AiOperation::Optimize, code).await;
                total_time += start.elapsed();
            }
            
            let avg_ms = total_time.as_secs_f64() * 1000.0 / iterations as f64;
            println!("  {} code ({} chars): {:.3}ms avg", 
                size_name, code.len(), avg_ms);
        }

        println!("\nCache Performance:\n");
        
        let code = "fn test() { 42 }";
        
        // Uncached
        service.clear_cache();
        let start = Instant::now();
        let _ = service.process(AiOperation::Explain, code).await;
        let uncached = start.elapsed();
        
        // Cached
        let start = Instant::now();
        let _ = service.process(AiOperation::Explain, code).await;
        let cached = start.elapsed();
        
        println!("  Uncached: {:.3}ms", uncached.as_secs_f64() * 1000.0);
        println!("  Cached:   {:.3}ms", cached.as_secs_f64() * 1000.0);
        println!("  Speedup:  {:.1}x", uncached.as_secs_f64() / cached.as_secs_f64());

        println!("\n{}", service.get_trace_summary());
    }
}
