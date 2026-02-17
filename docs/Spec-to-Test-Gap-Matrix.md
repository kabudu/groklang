# GrokLang Rust Spec-to-Test Gap Matrix (M2)

Date: 2026-02-17
Scope: Rust parser/type checker in `grok/`

Status legend:
- DONE: Implemented and covered by tests.
- PARTIAL: Implemented subset; needs additional semantics/coverage.
- TODO: Not implemented or not validated.

## Syntax and Grammar (`docs/Specifications/03-Syntax-Grammar.md`)

| Spec area | Status | Evidence (impl/tests) | Gap to close |
|---|---|---|---|
| Identifier rules (`letter { letter | digit | underscore }`) | DONE | `grok/src/parser.rs`, `grok/tests/parser.rs` | Add more negative tests for invalid starts/hyphens. |
| Arithmetic/comparison/logical precedence | PARTIAL | `grok/src/parser.rs`, `grok/tests/parser.rs` | Assignment semantics and full operator set still need spec-complete behavior. |
| Literals: int/float/string/byte string/bool | PARTIAL | `grok/src/parser.rs`, `grok/tests/parser.rs` | Char/typed suffixes still missing; raw + prefixed ints now supported. |
| Function/struct/enum/trait/actor definitions | PARTIAL | `grok/src/parser.rs`, `grok/tests/parser.rs` | Impl blocks, use/mod/module not supported. |
| Match expressions/arms/guards | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/type_checker.rs` | Or-pattern parsing is implemented; bool and enum exhaustiveness checks are in place; full structural exhaustiveness remains. |
| Concurrency syntax (`spawn`, `receive`, send) | PARTIAL | `grok/src/parser.rs`, `grok/tests/parser_actor.rs` | Align send syntax with final spec form and add diagnostics. |
| Macros (`macro_rules!`, macro call) | PARTIAL | `grok/src/parser.rs`, `grok/tests/parser.rs` | Expand grammar/validation beyond current subset. |
| Collections/closures/indexing/try operator | TODO | N/A | Implement parser + tests. |

## Type System (`docs/Specifications/02-Type-System-Specification.md`)

| Spec area | Status | Evidence (impl/tests) | Gap to close |
|---|---|---|---|
| Basic inference/unification | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | HM completeness and polymorphism generalization not complete. |
| Function types and call checking | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Better diagnostics and generic function constraints needed. |
| Struct literal/member typing | DONE | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Add negative-field and missing-field tests. |
| Match guard typing | DONE | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Add more guard-path tests. |
| Actor-related typing (`spawn`/`send`/`receive`) | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker_actor.rs` | Message type contracts and stronger actor API typing missing. |
| Traits/bounds/generics semantics | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Trait existence/duplicate checks and generic constructor/arity checks are in; bounds and impl semantics still missing. |
| Exhaustiveness checking | TODO | N/A | Implement for `match`. |

## M2 Batch 1 (Completed)
- Parser: identifier rule fixed to allow trailing digits.
- Parser: operator precedence chain added for logical/comparison/arithmetic operators.
- Parser: float/string/byte string literals added.
- Parser: type annotations expanded with unit/reference/generic forms.
- Type checker: function parameter type variables made unique per parameter.
- Type checker: match guards now enforced as boolean.
- Tests: parser/type-checker coverage added for the new behavior.

## M2 Batch 2 (Completed)
- Parser: expression grammar extended with assignment operators, bitwise operators, and shifts.
- Parser: expanded pattern forms added for tuple, struct, and enum patterns.
- Tests: positive and negative parser tests added for new operators and pattern forms.

## M2 Batch 3 (Completed)
- Parser: or-pattern grammar support (`p1 | p2 | p3`).
- Parser: prefixed integer literals (`0x`, `0b`, `0o`) and raw string literals (`r"..."`).
- Type checker: logical operators enforced as boolean-only.
- Type checker: or-pattern branch typing consistency checks.
- Tests: parser/type-checker coverage added for these behaviors.

## M2 Batch 4 (Completed)
- Type checker: bool and enum match exhaustiveness checks.
- Type checker: generic type constructor validation (unknown constructor + arity checks for built-ins).
- Tests: added enum exhaustiveness and generic validation coverage.

## M2 Batch 5 (Completed)
- Type checker: trait semantic validation (duplicate trait definitions and unknown trait type annotations).
- Parser: trait type annotation support (`trait Name`) in type positions.
- Tests: trait parsing and type-checker validation coverage added.

## M2 Batch 6 (Completed)
- AST/parser: `impl` block support (`impl Type {}` and `impl Trait for Type {}`).
- Type checker: impl semantic validation (unknown trait/type, missing required methods, duplicate impl methods).
- Parser/LSP: structured parse error (`ParseError`) with line/column/offset and direct LSP consumption.
- Type/borrow diagnostics: line/column enriched error messages for improved editor positioning.
- Tests: parser and type-checker coverage added for impl semantics and diagnostic metadata.

## Next M2 Batches (Planned)
1. Implement trait bounds and where-clause semantics checks.
2. Expand exhaustiveness beyond bool/enums (e.g., tuples/struct patterns where feasible).
3. Add broader structured diagnostics with AST-linked spans across more checker errors.
