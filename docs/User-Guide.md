# GrokLang User Guide

## Introduction

GrokLang is a modern, AI-enhanced programming language designed for safe, concurrent, and productive software development. It combines the best of Rust's memory safety, OCaml's type inference, Python's ease, and unique AI-powered features for code optimization, testing, and translation.

## Getting Started

### Installation

GrokLang requires Python 3.8+ and PLY. Install dependencies:

```bash
pip install ply
```

Clone the repository:

```bash
git clone https://github.com/yourorg/groklang.git
cd groklang
```

### Compiling and Running Programs

1. Write your GrokLang code in a `.grok` file, e.g., `hello.grok`.

2. Compile using the GrokLang compiler:

```bash
python3 -c "
from src.groklang.compiler import Compiler
compiler = Compiler()
with open('hello.grok', 'r') as f:
    code = f.read()
result = compiler.compile(code, target='vm')
# For VM execution
vm = result['vm']
vm.call_function('main', [])
"
```

For LLVM output:

```bash
python3 -c "
from src.groklang.compiler import Compiler
compiler = Compiler()
with open('hello.grok', 'r') as f:
    code = f.read()
result = compiler.compile(code, target='llvm')
print(result['llvm'])
"
```

## Hello World Example

The classic "Hello World" in GrokLang:

```groklang
fn main() {
    println("Hello, World!");
}
```

This defines a function `main` that prints a string. GrokLang uses `println` for output (assumed available via standard library).

To run:

1. Save as `hello.grok`.
2. Compile and execute as above.

## Basic Syntax

### Variables and Types

```groklang
fn example() {
    let x: i32 = 42;  // Explicit type
    let y = 3.14;     // Inferred type (f64)
    let name = "Grok"; // Inferred str
    let flag = true;   // Inferred bool

    mut z = 0;        // Mutable variable
    z = z + 1;
}
```

### Functions

```groklang
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: str) {
    println("Hello, " + name);
}
```

### Control Flow

```groklang
fn check_number(x: i32) -> str {
    if x > 0 {
        "positive"
    } else if x < 0 {
        "negative"
    } else {
        "zero"
    }
}

fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

match value {
    1 => "one",
    2 => "two",
    _ => "other",
}
```

### Structs and Enums

```groklang
struct Point {
    x: f64,
    y: f64,
}

enum Option<T> {
    Some(T),
    None,
}

fn use_struct() {
    let p = Point { x: 1.0, y: 2.0 };
    println(p.x);
}

fn use_enum() {
    let opt = Option::Some(42);
    match opt {
        Some(val) => println(val),
        None => println("None"),
    }
}
```

### Traits and Generics

```groklang
trait Printable {
    fn print(self);
}

impl Printable for i32 {
    fn print(self) {
        println(self);
    }
}

fn generic_add<T>(a: T, b: T) -> T where T: Add {
    a + b
}
```

### Concurrency

```groklang
fn concurrent_example() {
    let handle = spawn {
        println("Hello from thread!");
    };
    join(handle);
}

actor Counter {
    state: i32,

    fn increment(self) {
        self.state = self.state + 1;
    }

    fn get(self) -> i32 {
        self.state
    }
}
```

### AI Features

```groklang
#[ai_optimize(level: "high", target: "speed")]
fn slow_function(data: Vec<i32>) -> i32 {
    // Complex computation
    data.iter().sum()
}

#[ai_test(iterations: 100)]
fn my_function() {
    // Function to generate tests for
}

#[ai_translate(target_lang: "python")]
fn translate_me() {
    println("This will be translated to Python");
}
```

## Advanced Topics

- **Memory Safety**: Automatic borrow checking prevents data races.
- **FFI**: Call Python/C functions seamlessly.
- **Modules**: Use `use` for importing, `mod` for defining modules.
- **Closures**: Functional programming with `|params| body`.
- **Error Handling**: Use `?` for try-like behavior.

For more details, see:
- [Language Specification](docs/Specifications/03-Syntax-Grammar.md)
- [Implementation Roadmap](docs/Implementation-Roadmap.md)
- [AI Integration Guide](docs/Specifications/05-AI-Integration-Specification.md)