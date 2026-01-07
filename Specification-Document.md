GrokLang Spec v0.2:

- **Paradigm**: Multi-paradigm with AI core for code assist.
- **Syntax**: Rust-like: `fn main() { ai::optimize("loop here"); }`
- **AI Integration**: `ai` decorators for auto-refactor, debug, gen.
- **Generics/Traits**: `trait AiBound<T> where T: Optimizable;`
- **Concurrency**: Actors via `actor MyActor { fn behave() {} }` w/ AI deadlock scan.
- **Macros**: `macro_rules! ai_expand { ($e:expr) => { ai::expand($e) } }`
- **Memory**: Ownership + optional AI-GC: `let x = ai::alloc();`
- **Interop**: `#[ai_translate] extern "py" fn py_call(code: str);`
- **Testing**: `#[ai_test] fn test() { "assert eq" }` auto-fuzzes.

Implementation Plan:

1. Define lexer/parser in Python (use PLY).
2. Implement core syntax & AI stubs (mock LLM calls).
3. Add generics/traits via AST transforms.
4. Build concurrency runtime.
5. Integrate macros & memory mgmt.
6. Add interop/testing.
7. Iterate: Feed code snippets to me for simulation/refinement.
