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
- [x] **1.4 Validation & Testing**
    - [x] Unit tests for all grammar rules.
    - [x] Macro expansion tests.
    - [ ] Parser error recovery tests.

## Phase 2: Core Pipeline (Type Checker, IR, VM/LLVM)
- [/] **2.1 Type Checker**
    - [x] Hindley-Milner inference.
    - [x] Type unification.
    - [x] Borrow checker port (Exclusivity & Scoping).
- [x] **2.2 IR & LLVM Integration**
    - [x] Block-based IR generation.
    - [x] Support for function calls, macros, and structs.
    - [ ] Inkwell (LLVM) JIT integration.
- [/] **2.3 VM Expansion**
    - [x] Stack frames and Call Stack for recursion.
    - [x] Full arithmetic and comparison opcodes.
    - [ ] Object model and Garbage Collection (Heap-based).

## Phase 3: Advanced Features (AI, Concurrency, FFI)
- [ ] **3.1 Concurrency**
    - [ ] Actor supervision trees.
    - [ ] Deadlock detection.
- [ ] **3.2 AI Integration**
    - [ ] Real LLM integration (reqwest).
    - [ ] AI security sandboxing and post-analysis.
- [ ] **3.3 FFI**
    - [ ] PyO3 and bindgen integration.

## Phase 4: Ecosystem (CLI, LSP, Package Manager)
- [ ] **4.1 Tooling**
    - [ ] LSP completion, diagnostics, and hover.
    - [ ] CLI enhancements.
- [ ] **4.2 Productionization**
    - [ ] CI/CD pipelines.
    - [ ] Performance benchmarking (comparisons to Python).
