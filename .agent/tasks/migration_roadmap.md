# GrokLang Rust Migration Roadmap

## Phase 1: Foundation (Lexer, Parser, AST)
- [x] **1.1 Match Python AST Nodes**
    - [x] Update `ast.rs` with all node types (Structs, Enums, Actors, Traits, Match, etc.).
    - [x] Implement visitor/traversal traits (Skeletal).
- [x] **1.2 Complete Lexer**
    - [x] Add RawString and ByteString support.
    - [x] Ensure all operators and keywords are covered.
- [x] **1.3 Expand Parser**
    - [x] Implement Struct and Enum parsing.
    - [x] Implement Actor and Message passing syntax.
    - [x] Implement Trait and Impl parsing.
    - [x] Implement Match expressions and Patterns.
    - [x] Implement Loops (for, while) and Flow Control.
    - [x] Implement Macro definitions and expansion.
    - [x] Support for single-line comments (//).
- [x] **1.4 Validation & Testing**
    - [x] Unit tests for all grammar rules.
    - [x] Macro expansion tests.
    - [x] Parser error recovery tests (all_consuming validation).

## Phase 2: Core Pipeline (Type Checker, IR, VM/LLVM)
- [x] **2.1 Type Checker**
    - [x] Hindley-Milner inference.
    - [x] Type unification.
    - [x] Borrow checker port (Exclusivity & Scoping).
- [x] **2.2 Native Code Generation (Cranelift)**
    - [x] Integrate `cranelift` crates for JIT and native compilation.
    - [x] Implement JIT for arithmetic, recursion, and control flow.
    - [x] Optimized for Apple Silicon (aarch64) and x86_64.
- [/] **2.3 VM Expansion**
    - [x] Stack frames and Call Stack for recursion.
    - [x] Full arithmetic and comparison opcodes.
    - [x] Object model and Garbage Collection (Heap-based).
    - [x] Thread-safe asynchronous execution (Send + 'static).

## Phase 3: Advanced Features (AI, Concurrency, FFI)
- [x] **3.1 Concurrency**
    - [x] Core Actor primitives (Spawn, Send, Receive).
    - [x] Actor supervision trees (Foundational meta-data & status).
    - [x] Deadlock detection (Global stall detection).
- [x] **3.2 AI Integration**
    - [x] Real LLM integration (reqwest + OpenAI compatible).
    - [x] AI security sandboxing and post-analysis.
- [ ] **3.3 FFI**
    - [ ] PyO3 and bindgen integration.

## Phase 4: Ecosystem (CLI, LSP, Package Manager)
- [ ] **4.1 Tooling**
    - [ ] LSP completion, diagnostics, and hover.
    - [ ] CLI enhancements.
- [ ] **4.2 Productionization**
    - [ ] CI/CD pipelines.
    - [ ] Performance benchmarking (comparisons to Python).
