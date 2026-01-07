# GrokLang Architectural Decisions

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Status**: Final (Approved)

---

## Overview

This document records the core architectural decisions for GrokLang that will guide all implementation phases. These decisions were made to optimize for AI-first development while maintaining performance, safety, and developer experience.

---

## Decision 1: Type System Approach

**Selected**: Option C - Full Type Inference (ML-style)

### Rationale

- **AI-friendly**: Enables AI to infer complex types automatically, reducing boilerplate
- **Less explicit**: Reduces developer cognitive load compared to Rust-style annotations
- **Flexible**: Developers can add explicit annotations when needed for clarity
- **Aligns with AI-first philosophy**: Leverages AI as a core feature, not an afterthought

### Implementation Implications

- Type checker must support full bidirectional type inference (Hindley-Milner style)
- Optional explicit type annotations: `let x: i32 = 42;` or `let x = 42;` (both valid)
- AI can suggest inferred types for documentation purposes
- All type checking happens at compile time

### Related Components

- Type inference engine (Phase 2)
- AST type annotation nodes
- Error messages with type suggestions

---

## Decision 2: AI Integration Model

**Selected**: Option A (Primary) + Option B (Secondary with flag)

### Primary: Compile-time AI (Default)

- **Decorators expanded at parse/early-compile time**
- `#[ai_optimize]`, `#[ai_test]`, `#[ai_translate]` all expanded before code generation
- Deterministic, reproducible results
- Full validation of AI outputs before shipping

### Secondary: Runtime AI (Via Flag)

- **Available via `--enable-runtime-ai` compiler flag**
- For scenarios requiring dynamic optimization
- AI operations triggered at execution time
- Use cases: hot-path optimization, adaptive behavior

### Implementation Requirements

- Decorator processor in Phase 2 (AST transforms)
- AI service abstraction (local model or API)
- Output validation/safety checks
- Fallback behavior when AI fails (non-blocking compilation)
- Configuration system for AI service backend

### Switching Mechanism

```groklang
// Default: compile-time expansion
#[ai_optimize]
fn compute() { /* ... */ }

// Runtime expansion (requires --enable-runtime-ai flag)
#[ai_optimize(runtime)]
fn dynamic_compute() { /* ... */ }
```

---

## Decision 3: Memory Safety

**Selected**: Option A (Primary) + AI-GC (Secondary)

### Primary: Borrow Checker (Rust-style)

- **Strict ownership semantics** at compile time
- Move semantics for non-Copy types
- Borrow semantics: immutable borrows (`&T`) and mutable borrows (`&mut T`)
- Lifetime annotations when necessary (can be inferred by default)
- Zero-cost abstractions: no runtime overhead

### Secondary: Optional AI-GC

- **Supplementary garbage collection** for complex scenarios
- Triggered via `let x = ai::alloc();` or `#[ai_managed]`
- AI determines collection strategy (generational, concurrent, etc.)
- Used when borrow checker is too restrictive
- Trades some performance for convenience

### Safety Guarantees

- **No undefined behavior** from memory access
- **No data races** in multi-threaded code
- **No use-after-free** or double-free bugs
- Enforced at compile time by borrow checker

### Implementation Phases

- Phase 1: Basic ownership (move, copy)
- Phase 2: References and borrows
- Phase 2: Lifetime tracking
- Phase 3: AI-GC (optional allocation pool)

---

## Decision 4: Actor Model & Concurrency

**Selected**: Option A (Lightweight Threads) + Option B (Message-passing Actors) + AI Deadlock Scan

### Concurrency Models Supported

#### Model A: Lightweight Threads/Fibers

```groklang
spawn {
    // Runs in a lightweight thread
    compute_heavy_task();
}
```

- Mapped to OS threads or green threads (async tasks)
- Shared memory with synchronization
- Best for CPU-bound work

#### Model B: Message-passing Actors

```groklang
actor MyActor {
    fn behave() {
        receive(message) {
            // Handle message
        }
    }
}
```

- Isolated address spaces
- Asynchronous message passing
- Best for distributed/decoupled work

#### AI Deadlock Detection

- Compile-time static analysis of lock/receive patterns
- AI identifies potential circular wait conditions
- Suggests lock ordering to prevent deadlocks
- Runtime deadlock timeout warnings

### Implementation Approach

- Phase 4a: Basic spawn/join with Arc/Mutex
- Phase 4b: Actor framework with message queues
- Phase 4c: AI deadlock analysis in decorator processor
- Hybrid: Developers choose per-use-case

---

## Decision 5: Foreign Function Interface (FFI)

**Selected**: Language-agnostic FFI with Python as first target, Bidirectional

### Design Principles

1. **Language-agnostic**: Single calling convention works for any language
2. **Bidirectional**: Grok code calls Python AND Python calls Grok
3. **Type-safe**: Types are marshaled correctly across boundaries
4. **Error handling**: Exceptions propagate correctly

### Python as First Target

```groklang
#[ai_translate]
extern "py" {
    fn np_array_sum(arr: list) -> float;
}

// Calling from Grok -> Python
let result = np_array_sum(vec![1.0, 2.0, 3.0]);

// Calling from Python -> Grok (via export)
#[export("py")]
fn grok_compute(data: list) -> i32 {
    // Grok implementation
}
```

### Future Targets

- C/C++ (via C ABI)
- JavaScript/Node.js
- Go
- Rust
- Java (via JNI)

### Type Marshaling

- Automatic conversion for standard types (int, float, string, list, dict)
- Custom marshaling for complex types
- AI assists in marshaling logic generation

### Implementation Phases

- Phase 5a: Python FFI scaffold
- Phase 5b: Type marshaling layer
- Phase 5c: Bidirectional calling support
- Phase 5d: Language-agnostic infrastructure

---

## Implementation Priorities

Based on these decisions, the recommended implementation order:

1. **Phase 1**: Lexer/Parser (syntax foundation)
2. **Phase 2**: Type checker (Decision 1) + Decorator processor (Decision 2, compile-time path)
3. **Phase 3**: Code generation (IR to executable)
4. **Phase 4a**: Memory safety - borrow checker (Decision 3, primary path)
5. **Phase 4b**: Concurrency - lightweight threads (Decision 4, simpler path first)
6. **Phase 4c**: Actor model (Decision 4, second path)
7. **Phase 4d**: Deadlock detection (Decision 4, AI component)
8. **Phase 5**: FFI - Python support (Decision 5)

---

## Trade-offs and Rationale

### ML-style Inference vs Explicit Typing

- **Trade-off**: Less control vs. less boilerplate
- **Accepted**: AI can guide developers when needed

### Compile-time AI vs Runtime AI

- **Trade-off**: Less flexibility vs. more determinism
- **Accepted**: Runtime AI available as opt-in

### Borrow Checker vs Full GC

- **Trade-off**: Steeper learning curve vs. guaranteed safety
- **Accepted**: AI-GC escapehatches available

### Supporting Both Threads and Actors

- **Trade-off**: Implementation complexity vs. flexibility
- **Accepted**: Both models have legitimate use cases

### Bidirectional FFI

- **Trade-off**: More complex than one-way vs. better ecosystem integration
- **Accepted**: Necessary for practical adoption

---

## Revision History

| Version | Date        | Changes                    |
| ------- | ----------- | -------------------------- |
| 1.0     | Jan 7, 2026 | Initial approved decisions |
