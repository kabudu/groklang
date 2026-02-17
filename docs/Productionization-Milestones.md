# GrokLang Rust Productionization Milestones

Date: 2026-02-17
Scope: Rust implementation in `grok/`

## Definition of Production Done
- Language front-end passes full syntax/type/borrow test suite with deterministic diagnostics.
- Runtime executes core language/concurrency semantics with safety guarantees and regression coverage.
- Optimizer/JIT paths are correct, benchmarked, and guarded by equivalence tests.
- AI integration is optional, policy-gated, and deterministic under mock mode.
- FFI is stable (Python/C), documented, and validated in CI on supported targets.
- Tooling (CLI, LSP, packaging) is complete enough for day-to-day developer use.
- CI enforces correctness, performance budgets, and release artifact generation.

## Milestones

### M1: Front-End Hardening and CLI Baseline
Goals
- Make compile pipeline deterministic: parse -> macro expand -> type check -> borrow check -> IR.
- Add clear CLI modes for static checking vs execution.
- Remove ambiguous execution behavior (always executing first function).

Acceptance gates
- `grok check <file>` validates front-end and exits non-zero on errors.
- `grok compile <file>` produces IR generation success without implicit execution.
- `grok compile <file> --run --entry main` executes explicit entrypoint.
- Borrow checker integrated into compile/check flow.

### M2: Language Completeness (Spec Alignment)
Goals
- Close parser/type gaps against `docs/Specifications/03-Syntax-Grammar.md` and `02-Type-System-Specification.md`.
- Strengthen trait/generic and pattern exhaustiveness checks.

Acceptance gates
- Spec-mapped parser/type feature matrix reaches 100% for agreed MVP subset.
- Negative tests for invalid programs cover each grammar/type rule.

### M3: Runtime and Concurrency Safety
Goals
- Harden actor runtime semantics, supervision, deadlock behavior, and memory safety edges.
- Add deterministic runtime error taxonomy.

Acceptance gates
- Runtime/concurrency test suite passes with race/deadlock regression cases.
- Documented and stable actor lifecycle semantics.

### M4: Optimizer and JIT Correctness
Goals
- Replace placeholder optimizer passes with real transforms.
- Ensure optimized/JIT execution equivalence with baseline VM.

Acceptance gates
- Equivalence test corpus passes for baseline vs optimized/JIT.
- Performance benchmarks meet agreed thresholds.

### M5: AI Decorator Integration
Goals
- Wire language-level AI annotations/decorators into compiler flow.
- Enforce opt-in provider configuration, caching, and safety policy.

Acceptance gates
- Decorator behavior verified under mock and networked providers.
- Deterministic behavior in offline/mock CI mode.

### M6: FFI Stabilization
Goals
- Promote FFI from placeholders to stable Python/C interfaces.
- Define ABI/ownership/lifetime contracts and error boundaries.

Acceptance gates
- Python/C interop tests pass on supported platforms.
- Documentation includes supported types and failure modes.

### M7: LSP and Developer Experience
Goals
- Improve diagnostics precision, completions, and language features.
- Align editor experience with compiler behavior.

Acceptance gates
- LSP integration tests validate diagnostics and completion behavior.
- Diagnostic spans and messages are consistent with compiler output.

### M8: Release Engineering and CI
Goals
- Add reproducible build/release pipeline.
- Enforce lint/test/benchmark/security gates.

Acceptance gates
- Tagged release builds produce artifacts for target platforms.
- CI blocks merges on correctness/performance/security regressions.

## Immediate Execution Order
1. Execute M1 now.
2. Start M2 gap-closing with a spec-to-test checklist.
3. Parallelize M3/M4 once M2 parser/type interfaces stabilize.
