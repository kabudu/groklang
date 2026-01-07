# GrokLang Type System Specification

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Scope**: Complete type system specification with inference rules

---

## 1. Overview

GrokLang uses **ML-style full type inference** (Hindley-Milner system) with optional explicit type annotations. The type system is **sound** and **complete**, meaning:

- **Sound**: No ill-typed programs pass type checking
- **Complete**: Every well-typed program is accepted

Type checking occurs entirely at compile-time; types are erased after compilation unless needed for runtime behavior (e.g., trait objects).

---

## 2. Type Hierarchy

### 2.1 Primitive Types

```
bool     -- Boolean: true, false
i8       -- 8-bit signed integer
i16      -- 16-bit signed integer
i32      -- 32-bit signed integer (default integer literal type)
i64      -- 64-bit signed integer
i128     -- 128-bit signed integer
isize    -- Pointer-sized signed integer

u8       -- 8-bit unsigned integer
u16      -- 16-bit unsigned integer
u32      -- 32-bit unsigned integer
u64      -- 64-bit unsigned integer
u128     -- 128-bit unsigned integer
usize    -- Pointer-sized unsigned integer

f32      -- 32-bit floating point
f64      -- 64-bit floating point (default float literal type)

char     -- Unicode scalar value (32-bit)
str      -- String slice (immutable, unsized)
()       -- Unit type (empty tuple)
```

### 2.2 Compound Types

```
(T1, T2, ..., Tn)    -- Tuple of n types
[T; n]               -- Array of exactly n elements of type T
[T]                  -- Slice of elements of type T (unsized)

struct S { x: T1, y: T2, ... }    -- Product type
enum E { V1(T1), V2, ... }        -- Sum type / discriminated union

fn(T1, T2) -> R                   -- Function type
|T1, T2| -> R                     -- Closure type
```

### 2.3 Generic Types

```
T                          -- Type variable
Vec<T>                     -- Generic vector parameterized by T
Map<K, V>                  -- Generic map over keys K and values V
Option<T>                  -- Generic option type
Result<T, E>               -- Generic result type
```

### 2.4 Trait Types

```
dyn Trait                  -- Trait object (dynamic dispatch)
&dyn Trait                 -- Reference to trait object
Trait + Clone              -- Trait with multiple bounds
```

### 2.5 Reference Types

```
&T                         -- Immutable reference to T
&mut T                     -- Mutable reference to T
*const T                   -- Raw pointer (immutable)
*mut T                     -- Raw pointer (mutable)
```

---

## 3. Type Inference Algorithm

### 3.1 Hindley-Milner Type Inference

GrokLang implements a variant of Hindley-Milner type inference with support for:

1. **Bidirectional type checking**: Types flow both downward (application context) and upward (expression results)
2. **Constraint generation**: Type constraints collected during AST traversal
3. **Unification**: Constraints solved via union-find algorithm
4. **Generalization**: Inferred types generalized to universally-quantified types

### 3.2 Type Inference Rules

#### Expression Typing: `Γ ⊢ expr : T`

Reads as: "In environment Γ, expression `expr` has type `T`"

**Variables**:

```
Γ(x) = T
─────────
Γ ⊢ x : T
```

**Function Abstraction**:

```
Γ, x : T₁ ⊢ expr : T₂
──────────────────────────
Γ ⊢ fn(x : T₁) -> expr : T₁ → T₂
```

With inference (type annotations optional):

```
Γ, x : T₁ ⊢ expr : T₂       (T₁, T₂ inferred if omitted)
──────────────────────────
Γ ⊢ fn(x) -> expr : T₁ → T₂
```

**Function Application**:

```
Γ ⊢ f : T₁ → T₂     Γ ⊢ arg : T₁
─────────────────────────────────
Γ ⊢ f(arg) : T₂
```

**Let Binding**:

```
Γ ⊢ e₁ : T₁     Γ, x : ∀α. T₁ ⊢ e₂ : T₂
─────────────────────────────────────────
Γ ⊢ let x = e₁; e₂ : T₂
```

Where `∀α. T₁` means T₁ is generalized over free type variables.

**If Expression**:

```
Γ ⊢ cond : bool     Γ ⊢ then_expr : T     Γ ⊢ else_expr : T
─────────────────────────────────────────────────────────────
Γ ⊢ if cond { then_expr } else { else_expr } : T
```

**Match Expression**:

```
Γ ⊢ scrutinee : T     ∀pattern p_i: Γ, bindings ⊢ expr_i : T'
──────────────────────────────────────────────────────────────
Γ ⊢ match scrutinee { p₁ => e₁, ..., pₙ => eₙ } : T'
```

### 3.3 Type Variable Generation

When a type annotation is omitted, a fresh type variable is created:

```groklang
let x = 42;              // x : i32 (literal type inference)
let y = x + 1;           // y : i32 (from x's type)
let f = fn(a) -> a + 1;  // f : i32 -> i32
let g = fn(a) -> a;      // g : ∀T. T -> T (polymorphic identity)
```

---

## 4. Trait System

### 4.1 Trait Definitions

```groklang
trait Show {
    fn show(self) -> str;
}

trait Clone {
    fn clone(self) -> Self;
}

// Trait with generic parameter
trait Container<T> {
    fn len(self) -> usize;
    fn get(self, index: usize) -> Option<T>;
}

// Trait with multiple methods
trait Sequence<T> : Container<T> {
    fn append(mut self, item: T) -> ();
    fn iter(self) -> dyn Iterator<T>;
}
```

### 4.2 Trait Bounds

```groklang
// Single bound
fn print_it<T : Show>(x: T) -> () {
    println!(x.show());
}

// Multiple bounds (trait intersection)
fn clone_and_show<T : Clone + Show>(x: T) -> str {
    let y = x.clone();
    y.show()
}

// Where clause (equivalent)
fn print_it<T>(x: T)
    where T : Show
{
    println!(x.show());
}

// Trait bound on generic struct
struct Box<T : Clone> {
    value: T,
}
```

### 4.3 Trait Implementations

```groklang
impl Show for i32 {
    fn show(self) -> str {
        self.to_string()
    }
}

impl<T : Show> Show for Vec<T> {
    fn show(self) -> str {
        // Impl uses T's Show bound
        self.iter().map(fn(x) -> x.show()).join(", ")
    }
}

// Implement multiple traits
impl Clone for MyStruct {
    fn clone(self) -> Self { /* ... */ }
}
impl Show for MyStruct {
    fn show(self) -> str { /* ... */ }
}
```

### 4.4 Trait Objects and Dynamic Dispatch

```groklang
// Trait object: dynamic dispatch
let obj: dyn Show = 42;
println!(obj.show());

// Reference to trait object
let ref_obj: &dyn Show = &42;
println!(ref_obj.show());

// Collection of heterogeneous types
let items: Vec<dyn Show> = vec![
    42,
    "hello",
    3.14,
];
```

---

## 5. Generics and Specialization

### 5.1 Generic Structs

```groklang
struct Pair<T, U> {
    first: T,
    second: U,
}

struct Stack<T> {
    items: Vec<T>,
}

struct Tree<T : Clone> {
    value: T,
    left: Option<Box<Tree<T>>>,
    right: Option<Box<Tree<T>>>,
}
```

### 5.2 Generic Functions

```groklang
// Identity function
fn id<T>(x: T) -> T { x }

// Map function over iterables
fn map<T, U>(items: Vec<T>, f: fn(T) -> U) -> Vec<U> {
    items.iter().map(f).collect()
}

// Generic with trait bounds
fn max<T : Comparable>(a: T, b: T) -> T {
    if a.compare(&b) > 0 { a } else { b }
}
```

### 5.3 Monomorphization

At compile time, generic code is specialized for concrete types:

```groklang
let v1: Vec<i32> = vec![1, 2, 3];
let v2: Vec<str> = vec!["a", "b", "c"];

// Generates two versions:
// Vec<i32>::push(self, i32)
// Vec<str>::push(self, str)
```

**Specialization rules**:

- Each unique instantiation generates a separate copy
- Code size increases (trade-off accepted for performance)
- Monomorphization happens before code generation

### 5.4 Associated Types

```groklang
trait Iterator {
    type Item;  // Associated type

    fn next(mut self) -> Option<Self::Item>;
}

impl Iterator for Range {
    type Item = i32;

    fn next(mut self) -> Option<i32> {
        if self.current >= self.end {
            None
        } else {
            self.current += 1;
            Some(self.current - 1)
        }
    }
}
```

---

## 6. Pattern Matching and Exhaustiveness

### 6.1 Pattern Types

```groklang
// Literal patterns
match x {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("other"),
}

// Destructuring patterns
match point {
    (0, 0) => println!("origin"),
    (x, 0) => println!("on x-axis: {}", x),
    (0, y) => println!("on y-axis: {}", y),
    (x, y) => println!("at ({}, {})", x, y),
}

// Enum patterns
match maybe {
    Some(x) => println!("value: {}", x),
    None => println!("no value"),
}

// Guard expressions
match x {
    n if n > 0 => println!("positive"),
    n if n < 0 => println!("negative"),
    _ => println!("zero"),
}

// Or patterns
match x {
    1 | 2 | 3 => println!("small"),
    _ => println!("large"),
}
```

### 6.2 Exhaustiveness Checking

The compiler verifies that all patterns in a match are exhaustive:

```groklang
// ✓ Exhaustive
match option {
    Some(x) => { /* ... */ },
    None => { /* ... */ },
}

// ✓ Exhaustive with wildcard
match option {
    Some(x) => { /* ... */ },
    _ => { /* ... */ },
}

// ✗ Not exhaustive (compiler error)
match option {
    Some(x) => { /* ... */ },
    // Missing None case
}
```

---

## 7. Variance and Subtyping

### 7.1 Variance

Type constructors are either **covariant**, **contravariant**, or **invariant**:

```
T: covariant in T_i        if S <: T implies C<S> <: C<T>
T: contravariant in T_i    if S <: T implies C<T> <: C<S>
T: invariant in T_i        otherwise (not a subtype relationship)
```

**In GrokLang:**

- **Covariant**: `&T`, `Box<T>`, result types
- **Contravariant**: Function argument types
- **Invariant**: `&mut T`, mutable containers

### 7.2 Subtyping Rules

```
────────────
S <: S           (Reflexive)

S <: T    T <: U
────────────────
S <: U           (Transitive)

&S <: &T  when S = T  (Invariant for references)

fn(T₁) -> R <: fn(T₂) -> R  when T₂ <: T₁  (Contravariance in arguments)

Cov<S> <: Cov<T>  when S <: T  (Covariance in results)
```

---

## 8. Lifetime Tracking

### 8.1 Lifetime Variables

Lifetimes are explicitly tracked for references:

```groklang
// Lifetime parameter 'a
fn first<'a>(items: &'a [T]) -> &'a T {
    &items[0]
}

// Struct with lifetime
struct Ref<'a, T> {
    ptr: &'a T,
}

// Multiple lifetimes
fn combine<'a, 'b>(x: &'a T, y: &'b T) -> &'a T {
    if condition { x } else { x }
}
```

### 8.2 Lifetime Inference

Most lifetime annotations are **elided** (inferred):

```groklang
// ✓ Elision: Lifetime inferred
fn first(items: &[T]) -> &T {
    &items[0]
}

// Equivalent to (with lifetime made explicit):
fn first<'a>(items: &'a [T]) -> &'a T {
    &items[0]
}
```

**Elision rules**:

1. Single input lifetime → output lifetime
2. `&self` → output gets `self`'s lifetime
3. `&mut self` → output gets `self`'s lifetime

### 8.3 Lifetime Variance

```
'a <: 'b  if 'a outlives 'b (longer-lived)

// Covariant in output
fn() -> &'a T <: fn() -> &'b T  if 'a <: 'b

// Contravariant in input
fn(&'a T) -> () <: fn(&'b T) -> ()  if 'b <: 'a
```

---

## 9. Type Inference Examples

### 9.1 Simple Inference

```groklang
let x = 42;                     // x : i32
let y = 3.14;                   // y : f64
let z = x + 1;                  // z : i32
let s = "hello";                // s : str
```

### 9.2 Function Inference

```groklang
// Return type inferred from body
fn add(a, b) -> {
    a + b
}
// Inferred: fn(i32, i32) -> i32 (from literal types)

// Polymorphic function
fn id(x) -> {
    x
}
// Inferred: fn(∀T. T) -> T (from no constraints on x)

// Generic with constraints from usage
fn map(items, f) -> {
    items.iter().map(f).collect()
}
// Inferred after specialization: fn(Vec<T>, fn(T) -> U) -> Vec<U>
```

### 9.3 Complex Inference

```groklang
// Mutual recursion with inference
fn fib(n) -> {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}
// Inferred: fn(i32) -> i32

// Higher-order function
fn apply_twice(f, x) -> {
    f(f(x))
}
// Inferred: fn(fn(T) -> T, T) -> T

// With trait bounds from usage
fn process<T : Clone + Show>(item: T) -> str {
    item.clone().show()
}
```

---

## 10. Type Errors and Recovery

### 10.1 Error Categories

**Type mismatch**:

```groklang
let x: i32 = 3.14;  // Error: expected i32, found f64
```

**Unbound variable**:

```groklang
let y = x + 1;      // Error: cannot find 'x' in this scope
```

**Trait not implemented**:

```groklang
fn show<T : Show>(x: T) -> str { x.show() }
show(vec![]);       // Error: Vec doesn't implement Show
```

**Lifetime mismatch**:

```groklang
let r: &'static str = get_string();  // Error: lifetime mismatch
```

### 10.2 Error Messages with Suggestions

Compiler provides:

- **Location**: File, line, column
- **Message**: What went wrong
- **Suggestions**: How to fix it (AI-assisted)
- **Context**: Related code snippets

Example:

```
error[E0308]: mismatched types
  --> src/main.grok:5:18
   |
5  |     let x: i32 = 3.14;
   |                  ^^^^ expected i32, found f64
   |
note: Did you mean to use 42 instead of 3.14?
   = help: type annotations are inferred, try: let x = 42
```

---

## 11. Special Types

### 11.1 Never Type `!`

The never type represents functions that never return:

```groklang
fn exit(code: i32) -> ! {
    std::process::exit(code)
}

fn panic(msg: str) -> ! {
    println!(msg);
    unreachable!()
}
```

### 11.2 Unsized Types

Types without known size at compile time (must be behind pointers):

```groklang
[T]        -- Array slice (unsized)
str        -- String slice (unsized)
dyn Trait  -- Trait object (unsized)
```

### 11.3 Phantom Types

Types used only at compile time, erased at runtime:

```groklang
struct Id<T> {
    value: i32,
    // T is phantom: not stored, only used at compile time
    _phantom: PhantomData<T>,
}
```

---

## 12. Validation Criteria for Type System

Implementation must satisfy:

- [ ] Hindley-Milner inference works for 100% of expressions
- [ ] All type errors caught at compile time
- [ ] Exhaustiveness checking works for all patterns
- [ ] Trait bounds enforced correctly
- [ ] Generic monomorphization produces correct code
- [ ] Lifetime inference elision rules correct
- [ ] Error messages suggest fixes (with AI help)
- [ ] No type unsoundness (test suite for type safety)

---

## 13. Related Documents

- [01-Architecture-Overview.md](01-Architecture-Overview.md) - System architecture
- [03-Syntax-Grammar.md](03-Syntax-Grammar.md) - Concrete syntax
- [04-Runtime-Memory-Model.md](04-Runtime-Memory-Model.md) - Memory semantics
