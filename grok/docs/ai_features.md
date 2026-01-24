# GrokLang AI Features Guide

This document provides a comprehensive guide to GrokLang's AI integration capabilities, including configuration, usage examples, and performance benchmarks.

## Overview

GrokLang includes built-in AI assistance powered by large language models (LLMs). The AI module supports multiple providers and offers various code intelligence operations.

### Supported Providers

| Provider | API Base URL | Default Model |
|----------|--------------|---------------|
| **DeepSeek** | `https://api.deepseek.com` | `deepseek-chat` |
| **OpenAI** | `https://api.openai.com/v1` | `gpt-3.5-turbo` |
| **Mock** | N/A | `mock` (for testing) |

### AI Operations

| Operation | Description |
|-----------|-------------|
| `Optimize` | Optimize code for performance |
| `Explain` | Explain what code does in plain language |
| `Debug` | Find bugs and logical errors |
| `Refactor` | Improve code readability and structure |
| `GenerateTests` | Create comprehensive test cases |
| `SecurityAudit` | Check for security vulnerabilities |
| `DocumentCode` | Add documentation comments |
| `TranslateCode` | Translate between languages |

---

## Configuration

### Environment Variables

Configure GrokLang AI using these environment variables:

```bash
# Required: Choose provider
export GROK_AI_PROVIDER="deepseek"  # Options: "deepseek", "openai", "mock"

# Required: API key for the provider
export GROK_AI_KEY="your-api-key-here"

# Optional: Override default model
export GROK_AI_MODEL="deepseek-chat"

# Optional: Override base URL (for self-hosted endpoints)
export GROK_AI_BASE_URL="https://api.deepseek.com"
```

### DeepSeek Configuration

To use DeepSeek as your AI provider:

1. **Get an API Key**
   - Visit [DeepSeek Platform](https://platform.deepseek.com/)
   - Create an account and generate an API key
   
2. **Configure Environment**
   ```bash
   export GROK_AI_PROVIDER="deepseek"
   export GROK_AI_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxx"
   ```

3. **Optional: Specify Model**
   ```bash
   export GROK_AI_MODEL="deepseek-chat"  # Default
   # or
   export GROK_AI_MODEL="deepseek-coder"  # For code-specific tasks
   ```

### OpenAI Configuration

To use OpenAI:

```bash
export GROK_AI_PROVIDER="openai"
export GROK_AI_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxx"
export GROK_AI_MODEL="gpt-4"  # Optional, defaults to gpt-3.5-turbo
```

### Mock Mode (Testing)

For development without an API key:

```bash
export GROK_AI_PROVIDER="mock"
# No API key needed
```

---

## Usage Examples

### Rust API

```rust
use grok::ai::{AiConfig, AiOperation, AiService};

#[tokio::main]
async fn main() {
    // Create service with environment configuration
    let mut service = AiService::new();
    
    // Or configure explicitly
    let mut service = AiService::with_config(
        AiConfig::deepseek("your-api-key".to_string())
    );
    
    let code = r#"
        fn fibonacci(n) {
            if n < 2 { n }
            else { fibonacci(n-1) + fibonacci(n-2) }
        }
    "#;
    
    // Optimize code
    match service.process(AiOperation::Optimize, code).await {
        Ok(optimized) => println!("Optimized:\n{}", optimized),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Explain code
    match service.process(AiOperation::Explain, code).await {
        Ok(explanation) => println!("Explanation:\n{}", explanation),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Get trace report
    println!("{}", service.get_trace_report());
}
```

### All Operations Example

```rust
use grok::ai::{AiConfig, AiOperation, AiService};

async fn demonstrate_all_operations() {
    let mut service = AiService::new();
    
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
    
    let operations = vec![
        ("Optimization", AiOperation::Optimize),
        ("Explanation", AiOperation::Explain),
        ("Bug Detection", AiOperation::Debug),
        ("Refactoring", AiOperation::Refactor),
        ("Test Generation", AiOperation::GenerateTests),
        ("Security Audit", AiOperation::SecurityAudit),
        ("Documentation", AiOperation::DocumentCode),
    ];
    
    for (name, op) in operations {
        println!("\n=== {} ===", name);
        match service.process(op, sample_code).await {
            Ok(result) => println!("{}", result),
            Err(e) => println!("Error: {}", e),
        }
    }
    
    // Print performance summary
    println!("\n{}", service.get_trace_summary());
}
```

---

## Trace Log & Benchmarks

### Trace Log Format

Every AI operation is logged with detailed metrics:

```
=== AI Operation Trace Log ===

Operation #1
- Type: optimize
- Provider: DeepSeek (deepseek-chat)
- Latency: 1250ms
- Cached: false
- Success: true
- Input Tokens: ~87
- Output Tokens: ~95

Operation #2
- Type: explain
- Provider: DeepSeek (deepseek-chat)
- Latency: 0ms
- Cached: true
- Success: true
- Input Tokens: ~87
- Output Tokens: ~150

AI Trace Summary:
- Total Operations: 2
- Successful: 2 (100.0%)
- Cached: 1 (50.0%)
- Total Latency: 1250ms
- Average Latency: 625ms
```

### Performance Benchmarks (Mock Mode)

These benchmarks demonstrate the overhead of the AI service without network latency:

| Metric | Value |
|--------|-------|
| Small code optimization | ~12ms |
| Medium code optimization | ~12ms |
| Large code optimization | ~12ms |
| Cache hit latency | ~0.009ms |
| Cache speedup | **1375x** |

### Performance Benchmarks (With LLM)

Typical latencies when using real LLM providers:

| Operation | DeepSeek | OpenAI GPT-3.5 |
|-----------|----------|----------------|
| Optimize | 800-1500ms | 500-1000ms |
| Explain | 1000-2000ms | 600-1200ms |
| Debug | 1200-2500ms | 700-1400ms |
| Security Audit | 1500-3000ms | 800-1600ms |

*Note: Latencies vary based on code size, network conditions, and server load.*

---

## Security Features

### Output Validation

All AI outputs are validated before being returned:

```rust
// Blocked patterns
let dangerous_patterns = [
    "eval(",
    "exec(",
    "system(",
    "os.popen",
    "__import__",
    "subprocess",
    "rm -rf",
    "DROP TABLE",
    "DELETE FROM",
    "; DROP",
];
```

### Security Audit Results

The security audit operation checks for:
- SQL injection vulnerabilities
- Hardcoded credentials
- Command injection risks
- Unsafe function calls
- Input validation issues

---

## Running the Examples

### Run AI Demo Tests

```bash
# With mock mode (no API key needed)
cargo test ai_demo --release -- --nocapture

# With DeepSeek
export GROK_AI_PROVIDER="deepseek"
export GROK_AI_KEY="your-key"
cargo test ai_demo --release -- --nocapture
```

### Sample Test Output

```
======================================
GrokLang AI Features Demonstration
======================================

Provider: Mock
Model: mock

>>> Running: Code Optimization <<<
✓ Success (11.62ms)
Output preview: // Optimized version fn fibonacci(n) { if n < 2 { return n }...

>>> Running: Code Explanation <<<
✓ Success (12.02ms)
Output preview: This code performs the following: 1. Defines a function...

>>> Running: Security Audit <<<
✓ Success (12.05ms)
Output preview: Security Audit Report: - No SQL injection vulnerabilities...

>>> Testing Cache <<<
Cached call time: 0.006ms

AI Trace Summary:
- Total Operations: 8
- Successful: 8 (100.0%)
- Cached: 1 (12.5%)
- Total Latency: 82ms
- Average Latency: 10ms
```

---

## Best Practices

1. **Use Caching**: The AI service caches results automatically. Identical requests return instantly.

2. **Choose the Right Operation**: Use specific operations rather than generic prompts for better results.

3. **Handle Errors**: Always handle potential API errors gracefully.

4. **Monitor Traces**: Use the trace log to identify performance bottlenecks.

5. **Validate Output**: While the service includes security checks, always review AI-generated code before use.

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "API key not configured" | Set `GROK_AI_KEY` environment variable |
| "Network error" | Check internet connection and API endpoint |
| "AI output failed security check" | The AI generated potentially dangerous code |
| Slow responses | Try a different model or check provider status |

### Debug Mode

Enable detailed logging:

```rust
let service = AiService::new();
// After operations...
println!("{}", service.get_trace_report());
```

---

## API Reference

### `AiConfig`

```rust
pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
    pub timeout_secs: u64,
    pub max_tokens: u32,
}

impl AiConfig {
    fn openai(api_key: String) -> Self;
    fn deepseek(api_key: String) -> Self;
    fn mock() -> Self;
    fn from_env() -> Self;
}
```

### `AiOperation`

```rust
pub enum AiOperation {
    Optimize,
    Explain,
    Debug,
    Refactor,
    GenerateTests,
    SecurityAudit,
    DocumentCode,
    TranslateCode,
    Custom(String),
}
```

### `AiService`

```rust
impl AiService {
    fn new() -> Self;
    fn with_config(config: AiConfig) -> Self;
    async fn process(&mut self, op: AiOperation, code: &str) -> Result<String, String>;
    fn is_safe(&self, output: &str) -> bool;
    fn get_trace_summary(&self) -> String;
    fn get_trace_report(&self) -> String;
    fn clear_cache(&mut self);
}
```

---

## Related Documentation

- [Performance Benchmarks](./performance_benchmarks.md) - VM performance comparisons
- [GrokLang README](../README.md) - Main project documentation
