# GrokLang Rust Spec-to-Test Gap Matrix (M2)

Date: 2026-02-26
Scope: Rust parser/type checker in `grok/`

Status legend:
- DONE: Implemented and covered by tests.
- PARTIAL: Implemented subset; needs additional semantics/coverage.
- TODO: Not implemented or not validated.

## Syntax and Grammar (`docs/Specifications/03-Syntax-Grammar.md`)

| Spec area | Status | Evidence (impl/tests) | Gap to close |
|---|---|---|---|
| Identifier rules (`letter { letter | digit | underscore }`) | DONE | `grok/src/parser.rs`, `grok/tests/parser.rs` | Add more negative tests for invalid starts/hyphens. |
| Arithmetic/comparison/logical precedence | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/type_checker.rs` | Assignment semantics and full operator set still need spec-complete behavior; unary logical-not/numeric-negation/bitwise-not/plus, assignment-target validity checks, and reference/deref unary semantics are now validated in parser/checker coverage. |
| Literals: int/float/string/byte string/bool | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/type_checker.rs` | Raw/prefixed ints, typed numeric suffixes, char literals, byte-string literal typing (`Vec<u8>`), expanded primitive type annotation parsing (including `str` and full int-width primitives), and escaped string/byte-string/char parsing are supported; additional literal categories remain. |
| Function/struct/enum/trait/actor definitions | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/type_checker.rs` | Impl blocks, trait bounds, function where-clauses, richer use/module forms (grouped, glob, module declaration), internal use alias/group/glob name materialization for checker resolution, module declaration/definition coherence checks, and `pub fn` visibility-gated module imports are supported; full module visibility/export semantics remain pending. |
| Match expressions/arms/guards | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/type_checker.rs` | Or-pattern parsing is implemented; bool/enum plus finite tuple/struct/payload-sensitive coverage (including nested finite enum payload decomposition, recursive payload/struct finite-subdomain decomposition, nested recursive-enum skeleton splitting, and bounded deeper recursive-enum unfolding for nested splits) exists, with guarded arms excluded from coverage; broader general recursive coverage remains. |
| Concurrency syntax (`spawn`, `receive`, send) | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser_actor.rs`, `grok/tests/type_checker_actor.rs` | Send/receive parsing is in and actor send message typing is enforced from receive-pattern inference; richer actor protocol/API semantics remain. |
| Macros (`macro_rules!`, macro call) | PARTIAL | `grok/src/parser.rs`, `grok/src/macro_expander.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/macro_expander.rs`, `grok/tests/type_checker.rs` | Parser support, macro expansion integration in the type-check pipeline, and unknown-macro diagnostics are in; full macro-system parity with production macro semantics remains. |
| Collections/closures/indexing/try operator | PARTIAL | `grok/src/parser.rs`, `grok/src/type_checker.rs`, `grok/tests/parser.rs`, `grok/tests/type_checker.rs` | Closures, tuple literals, indexing, array literals, byte-string literal collection typing (`Vec<u8>`), and try-operator are implemented; broader collection forms remain missing. |

## Type System (`docs/Specifications/02-Type-System-Specification.md`)

| Spec area | Status | Evidence (impl/tests) | Gap to close |
|---|---|---|---|
| Basic inference/unification | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Tuple/reference-aware unification and polymorphic call-site instantiation are in; full HM generalization/inference completeness is still not complete. |
| Function types and call checking | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Call checking includes where-bound trait enforcement for concrete mapped type variables, and unification/impl-signature mismatch diagnostics now carry AST-linked spans; broader polymorphic constraint solving remains. |
| Struct literal/member typing | DONE | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Field existence/missing/duplicate checks are covered; continue broadening diagnostics. |
| Match guard typing | DONE | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Add more guard-path tests. |
| Actor-related typing (`spawn`/`send`/`receive`) | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker_actor.rs` | Receive-pattern message typing is enforced for sends to known actor types; richer actor protocol/API contracts remain. |
| Traits/bounds/generics semantics | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Trait existence/duplicate/bounds checks, impl completeness/signatures, function where-clause checks, call-site trait-bound enforcement, generic-constructor bound checking, repeated-parameter generic-impl matching checks, multi-parameter impl-bound solving, subtrait-implied bound satisfaction, and blanket generic impl solving (`impl<T: Bound> Trait for T`) are in; richer generic constraints remain. |
| Exhaustiveness checking | PARTIAL | `grok/src/type_checker.rs`, `grok/tests/type_checker.rs` | Bool/enum, finite tuple/struct, nested finite enum-payload, and recursive payload/struct decomposition over finite subdomains are implemented; recursive/non-finite enum payload variants without finite decomposition require catch-all payload patterns, and non-finite recursive struct matches require wildcard arms when finite decomposition is unavailable; full general recursive decomposition remains. |

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

## M2 Batch 7 (Completed)
- Type checker: trait impl signature compatibility checks (arity, parameter types, return type).
- Parser/type checker: trait bounds parsing and semantic validation (`trait A: B + C`).
- Type checker: stricter struct literal checking (unknown/missing/duplicate fields).
- Tests: parser and type-checker coverage for trait bounds/signature mismatches and struct literal negatives.

## M2 Batch 8 (Completed)
- AST/parser: index access expressions added (`expr[index]`) in postfix grammar.
- Type checker: index type enforcement (`i32` index) and `Vec<T>` element typing for indexed access.
- Tests: parser and type-checker coverage for index access plus negative index-type checks.

## M2 Batch 9 (Completed)
- AST/parser: `char` literal support added.
- Parser/type annotations: `char` type keyword support added.
- Type checker: `char` literal type inference support.
- Tests: parser and type-checker coverage for `char` literals.

## M2 Batch 10 (Completed)
- Parser: typed numeric suffix support for integer and float literals (`i32`/`i64`, `f32`/`f64`).
- Tests: parser coverage expanded for typed-suffix numeric literals.

## M2 Batch 11 (Completed)
- Parser: function `where` clause grammar support (single/multi bound entries).
- Type checker: where-clause semantic validation for known traits and signature-referenced type variables.
- Tests: parser/type-checker coverage for valid and invalid where-clause scenarios.

## M2 Batch 12 (Completed)
- AST/parser: `use` declarations and `mod` blocks added.
- Type checker: module traversal and use-declaration no-op typing integration.
- Tests: parser coverage for `use`/`mod` syntax.

## M2 Batch 13 (Completed)
- AST/parser: postfix try-operator support (`expr?`).
- Type checker: try-operator semantics for `Option<T>` and `Result<T, E>`.
- Tests: parser/type-checker coverage for valid/invalid try usage.

## M2 Batch 14 (Completed)
- AST/parser: array literal expressions (`[e1, e2, ...]`) added.
- Type checker: homogeneous element-type validation and array literal typing as `Vec<T>`.
- Macro expander: array literal traversal support.
- Tests: parser/type-checker coverage for array literals and mixed-type failures.

## M2 Batch 15 (Completed)
- AST/parser: closure expression support added (`|args| expr`, `move |args| -> T { ... }`).
- Type checker: closure typing added, yielding function types from closure parameters/body.
- Tests: parser/type-checker coverage for closure parsing and typed closure call usage.

## M2 Batch 16 (Completed)
- Parser: fixed assignment operator parsing so `=>` in match arms is not misparsed as `=`.
- Type checker: exhaustiveness now ignores guarded arms for bool/enum coverage.
- Diagnostics: bool/enum non-exhaustive match errors include line/col context.
- Tests: guarded-match regressions added for bool and enum exhaustiveness behavior.

## M2 Batch 17 (Completed)
- Parser/AST: richer import/module forms (`use ...::{...}`, `use ...::*`, `mod name;`) added.
- Parser/AST: tuple literal expressions added.
- Tests: parser coverage added for grouped/glob use, module declarations, and tuple literals.

## M2 Batch 18 (Completed)
- Type checker: tuple type support in inference/unification/substitution.
- Type checker: tuple pattern typing and tuple-bool exhaustiveness checking.
- Type checker: struct-bool pattern exhaustiveness checking.
- Tests: added exhaustive/non-exhaustive tuple and struct bool match coverage.

## M2 Batch 19 (Completed)
- Type checker: internal module/use path indexing and semantic validation for use declarations.
- Type checker: support for richer use forms validation (grouped imports, glob imports, declarations).
- Tests: pass/fail coverage for internal use-path resolution.

## M2 Batch 20 (Completed)
- Type checker: call-site enforcement of function `where` trait bounds via trait-impl lookup.
- Type checker: finite-domain tuple exhaustiveness expanded to mixed tuple(bool/enum) patterns.
- Tests: where-bound call-site pass/fail coverage and tuple(enum,bool) exhaustiveness coverage.

## M2 Batch 21 (Completed)
- Type checker: payload-sensitive enum exhaustiveness for finite payload domains (e.g., bool payload variants).
- Type checker: polymorphic function call instantiation (fresh type vars per call for global function schemes).
- Tests: enum payload exhaustive/non-exhaustive coverage and polymorphic independent-call instantiation coverage.

## M2 Batch 22 (Completed)
- Parser/type annotations: tuple type annotation support (`(T1, T2, ...)`) added.
- Type checker: finite enum-payload exhaustiveness extended to nested tuple finite domains.
- Type checker: generic-constructor trait-bound enforcement at call sites (e.g., impl for `Vec` applies to `Vec<T>` in where-bound checking).
- Tests: added enum tuple-payload exhaustive/non-exhaustive coverage and generic-constructor bound pass/fail coverage.

## M2 Batch 23 (Completed)
- Type checker: finite enum-payload exhaustiveness extended to nested finite enum payload decomposition (with recursion guard for non-finite cycles).
- Type checker: generic impl solving tightened for repeated type parameters in impl heads (e.g., `HashMap<T, T>` rejects mismatched concrete args).
- Tests: added nested enum-payload exhaustive/non-exhaustive coverage and repeated-parameter impl-bound pass/fail coverage.

## M2 Batch 24 (Completed)
- Type checker: non-finite/recursive enum payload variants now require catch-all payload patterns (`_`/identifier/variant payload wildcard) for exhaustive matching when no top-level wildcard arm exists.
- Type checker: definition-time diagnostics expanded with line/column context across trait/impl duplicate and unknown/bad-bound paths; where-bound call-site failures now include source location.
- Type checker: impl generic-bound validation tightened to reject bounds for type params not present in impl type-head params.
- Tests: added recursive non-finite payload pass/fail coverage, two-parameter generic impl-bound solving pass/fail coverage, and invalid impl-bound-param diagnostics coverage.

## M2 Batch 25 (Completed)
- Type checker: recursive payload decomposition now supports finite-subdomain reasoning by collapsing recursive cycles to bounded wildcard domain atoms, enabling exhaustive checks like recursive list payloads split by finite bool fields.
- Type checker: domain encoding/decoding for tuple/struct payload coverage made escape-safe for nested recursive payload values.
- Type checker: use-path validation diagnostics now include line/column context.
- Tests: added recursive finite-subdomain decomposition pass/fail coverage and strengthened use-path diagnostic position coverage.

## M2 Batch 26 (Completed)
- Type checker: finite-domain struct exhaustiveness generalized beyond bool-only structs to arbitrary finite field domains via domain-based structural coverage.
- Type checker: named-type finite-domain expansion now has recursion guards (not only enum-cycle guards), enabling recursive struct decomposition without infinite expansion.
- Type checker: removed obsolete bool-only struct coverage helpers after migration to generalized structural coverage.
- Tests: added recursive struct finite-subdomain decomposition pass/fail coverage.

## M2 Batch 27 (Completed)
- Type checker: trait-bound solving expanded with trait implication semantics (`Sub: Base` means `Sub` implies `Base`) for where-bound satisfaction.
- Type checker: generic impl matching now respects implied supertrait satisfaction (not only exact trait-name matches) when evaluating trait requirements.
- Tests: added transitive trait-bound pass/fail coverage for concrete and generic impl scenarios.

## M2 Batch 28 (Completed)
- Type checker: AST-linked diagnostic precision expanded for type-annotation validation errors (unknown trait types, unknown generic constructors, generic arity mismatches, unknown struct types) by propagating source spans through validation paths.
- Tests: added explicit position-assertion coverage for unknown trait-type and unknown generic-constructor validation errors.

## M2 Batch 29 (Completed)
- Type checker: where-bound call-site solving no longer silently skips unresolved bound variables; unresolved type variables in where-bounds now produce explicit call-site errors with source location.
- Tests: added unresolved where-bound variable regression coverage (pure-return type-var and partially unresolved multi-var scenarios).

## M2 Batch 30 (Completed)
- Type checker: internal `use` alias/group/glob imports now materialize bound names into checker globals, enabling alias-imported symbol resolution during type checking (not only path validation).
- Tests: added alias-imported function-call and grouped alias import pass coverage.

## M2 Batch 31 (Completed)
- Type checker: actor message typing strengthened by inferring receive-pattern message types and enforcing them for sends to known actor targets.
- Type checker: macro-call semantic validation added; unknown macro calls now fail with source-position diagnostics.
- Tests: added actor send message-type pass/fail coverage and macro known/unknown-call validation coverage.

## M2 Batch 32 (Completed)
- Type checker: macro expansion is now integrated directly into the type-check pipeline (`MacroExpander` runs before definition/constraint passes), so expanded macro expressions participate in regular type checking semantics.
- Type checker: recursive enum finite-domain decomposition now preserves one-step recursive skeleton shape during cycle handling (enabling nested recursive pattern split checks like `S(Z)` vs `S(S(_))`).
- Tests: added macro-expansion typing pass/fail coverage and recursive-enum nested decomposition exhaustive/non-exhaustive coverage.

## M2 Batch 33 (Completed)
- Type checker: constraint/unification diagnostics now preserve and report AST-linked source spans (line/col) for type mismatch, tuple/function arity mismatch, and recursive-type-detection paths.
- Type checker: trait-impl signature mismatch diagnostics (arity, parameter type, return type) now include impl-block source location.
- Tests: added span-assertion coverage for unification mismatch diagnostics and strengthened impl-signature mismatch diagnostics coverage.

## M2 Batch 34 (Completed)
- Type checker: blanket generic trait impl semantics added for bound type-parameter targets (e.g., `impl<T: Show> Marker for T`), including definition-time validation and call-site trait-bound solving.
- Type checker: impl generic-bound parameter validation now allows blanket impl bound parameters tied to the impl target type parameter.
- Tests: added where-bound pass/fail coverage for blanket generic impl solving.

## M2 Batch 35 (Completed)
- Type checker: recursive enum finite-domain handling now supports bounded deeper unfolding during cycle handling, enabling deeper nested recursive pattern split exhaustiveness checks beyond one-step skeletons.
- Tests: added deeper recursive nested split exhaustive/non-exhaustive coverage for `enum Nat { Z, S(Nat) }`.

## M2 Batch 36 (Completed)
- Type checker: module semantic coherence validation added for duplicate module declarations, duplicate module definitions, and mixed declaration+inline-definition conflicts.
- Diagnostics: module coherence failures include source line/column.
- Tests: added module coherence fail-path coverage with position assertions.

## M2 Batch 37 (Completed)
- Type checker: byte-string literals now type as `Vec<u8>` (instead of falling through to untyped/default paths).
- Parser/type annotations: primitive type parsing broadened for full scalar set (`i8/i16/i32/i64/i128/isize`, `u8/u16/u32/u64/u128/usize`, `f32/f64`, `char`, `bool`, `str`, `String`) so primitives are no longer mis-modeled as type variables in annotations.
- Tests: added byte-string typing pass/fail coverage (`Vec<u8>` acceptance and `str` mismatch rejection).

## M2 Batch 38 (Completed)
- Type checker: unary operator semantics tightened for `!` and `-` (`!` enforces boolean operand/result; `-` rejects non-numeric operands).
- Tests: added unary-operator pass/fail coverage (`!true`, `!1`, `-true`).

## M2 Batch 39 (Completed)
- Parser: unary bitwise-not operator (`~`) added to unary expression grammar.
- Type checker: unary `~` semantics added (integral-only), and assignment/compound-assignment now reject non-assignable targets (non-lvalues).
- Diagnostics: invalid assignment target errors include line/column context.
- Tests: added parser coverage for unary `~`, type-checker pass/fail coverage for unary `~`, and invalid-assignment-target negative coverage.

## M2 Batch 40 (Completed)
- Parser: escaped literal handling added for string, byte-string, and char literals (`\\`, `\"`, `\n`, `\r`, `\t`, `\0`, and escaped quote for chars).
- Tests: added parser coverage for escaped string/byte-string and escaped char literals.

## M2 Batch 41 (Completed)
- Parser/type checker: unary plus operator support added (`+expr`) with numeric-only type enforcement.
- Type checker: reference/deref unary semantics tightened (`&`/`&mut` produce reference types; `*` requires references) and reference-aware unification/substitution/occurs-check support added.
- Tests: added parser unary-plus coverage and type-checker reference/deref plus unary-plus pass/fail coverage.

## M2 Batch 42 (Completed)
- Parser: `pub fn` declarations are now parsed and encoded for checker visibility handling.
- Type checker: module import validation/materialization now enforces function visibility for internal imports (`use m::f`, grouped imports, and glob imports) based on `pub fn`.
- Tests: added parser coverage for `pub fn`, updated use-import pass cases to public functions, and added private-import negative coverage with line/col assertions.

## M2 Batch 43 (Completed)
- Type checker: exhaustiveness checking for non-finite recursive structs now enforces a wildcard arm when finite structural decomposition is not possible.
- Tests: added non-finite recursive struct wildcard-required pass/fail coverage.

## Next M2 Batches (Planned)
1. Extend exhaustiveness to deeper structural/data-carrying patterns beyond current finite-domain coverage (especially recursive/non-finite domains).
2. Add broader structured diagnostics with AST-linked spans across more checker errors.
3. Extend generics and trait constraints beyond current constructor/where-bound checks (e.g., richer parameterized bound forms and broader constraint solving).
