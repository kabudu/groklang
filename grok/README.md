# GrokLang Rust Implementation

The Rust implementation of the GrokLang programming language, offering high performance, memory safety, and modern language features.

## Overview

GrokLang is a modern programming language featuring:
- **Actor-based concurrency** with built-in message passing
- **Pattern matching** with exhaustive checking
- **Type inference** with Hindley-Milner algorithm
- **AI-powered code assistance** with multiple LLM providers
- **Native code generation** via Cranelift JIT

## Quick Start

```bash
# Build
cargo build --release

# Run a GrokLang file
cargo run --release -- run examples/hello.grok

# Start the REPL
cargo run --release -- repl

# Compile to native
cargo run --release -- compile myfile.grok
```

## Documentation

### Core Documentation
- [AI Features Guide](docs/ai_features.md) - AI-powered code assistance with DeepSeek/OpenAI
- [Performance Benchmarks](docs/performance_benchmarks.md) - VM performance comparisons and optimizations

### Language Features
- Functions and closures
- Structs and enums with pattern matching
- Actors with supervision trees
- Traits and generic types
- Macros for metaprogramming

## AI Integration

GrokLang includes built-in AI assistance. Configure with environment variables:

```bash
# Use DeepSeek
export GROK_AI_PROVIDER="deepseek"
export GROK_AI_KEY="your-api-key"

# Or OpenAI
export GROK_AI_PROVIDER="openai"
export GROK_AI_KEY="your-api-key"
```

Available AI operations:
- **Optimize** - Improve code performance
- **Explain** - Get plain-language explanations
- **Debug** - Find bugs and issues
- **Refactor** - Improve code structure
- **GenerateTests** - Create test cases
- **SecurityAudit** - Check for vulnerabilities

See [AI Features Guide](docs/ai_features.md) for complete documentation.

## Performance

The Rust implementation includes several optimizations:
- **Bytecode Specialization** - Type-specialized opcodes (33x faster locals)
- **Inline Caching** - Cached function and field lookups
- **Tail Call Optimization** - Constant-space recursion
- **Hot Path Detection** - Automatic JIT compilation triggers

See [Performance Benchmarks](docs/performance_benchmarks.md) for detailed comparisons.

## Testing

```bash
# Run all tests
cargo test --release

# Run AI demo tests
cargo test ai_demo --release -- --nocapture

# Run optimization benchmarks
cargo test optimization_benchmarks --release -- --nocapture
```

## Project Structure

```
grok/
├── src/
│   ├── ai.rs           # AI integration (DeepSeek, OpenAI)
│   ├── ast.rs          # Abstract syntax tree
│   ├── parser.rs       # Parser implementation
│   ├── type_checker.rs # Type inference and checking
│   ├── ir.rs           # Intermediate representation
│   ├── vm.rs           # Virtual machine
│   ├── optimizations.rs # VM optimizations
│   ├── jit.rs          # JIT compilation (Cranelift)
│   └── ...
├── tests/              # Test suites
├── benchmarks/         # Performance benchmarks
└── docs/               # Documentation
```

## License

See the root [LICENSE](../LICENSE) file.
