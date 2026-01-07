# GrokLang Module System Specification

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Scope**: Module organization, imports, namespacing, visibility

---

## 1. Module System Overview

GrokLang uses a **hierarchical module system** with:

- **File-based modules**: Each file is a module
- **Directory-based packages**: Directories organize modules
- **Explicit imports**: All dependencies explicitly declared
- **Public/private visibility**: Fine-grained access control
- **Re-exports**: Ability to expose nested modules

---

## 2. Module Hierarchy

### 2.1 File Structure

```
my_project/
├── src/
│   ├── lib.grok         // Root of library
│   ├── main.grok        // Entry point
│   ├── math/
│   │   ├── algebra.grok
│   │   └── geometry.grok
│   ├── io/
│   │   ├── file.grok
│   │   └── network.grok
│   └── utils.grok
├── tests/
│   └── integration.grok
└── grok.toml           // Package manifest
```

### 2.2 Module Paths

**Absolute paths** (from crate root):

```groklang
use my_project::math::algebra;
use my_project::io::file::read;
use my_project::utils::helper;
```

**Relative paths** (from current module):

```groklang
use super::utils;           // Parent module
use crate::math::algebra;   // Crate root
```

### 2.3 Module Declaration

**File-based (implicit)**:

```
src/math/algebra.grok exists
→ Module path: my_project::math::algebra
```

**Inline (explicit)**:

```groklang
mod math {
    mod algebra {
        // definitions
    }
    mod geometry {
        // definitions
    }
}
```

---

## 3. Visibility Rules

### 3.1 Privacy by Default

Items private by default, made public explicitly:

```groklang
// Private (default)
fn private_helper(x: i32) -> i32 {
    x * 2
}

// Public
pub fn public_function(x: i32) -> i32 {
    private_helper(x)
}

// Public struct, private field
pub struct Point {
    pub x: f64,
    y: f64,  // Private
}

// Public enum, all variants public
pub enum Result {
    Ok(T),
    Err(E),
}
```

### 3.2 Visibility Modifiers

```groklang
pub                      // Public to all

pub(crate)              // Public within crate only

pub(super)              // Public to parent module only

pub(in path::to)        // Public only to specific path
pub(in crate::special)

// Module can have visibility too
pub mod public_module { }
mod private_module { }
```

### 3.3 Re-exports

Make private items available as if public:

```groklang
// In file: src/io/mod.grok
mod file;
mod network;

// Re-export to consumers
pub use self::file::read;
pub use self::network::*;

// Now consumers can:
use my_project::io::read;           // From re-export
use my_project::io::connect;        // From re-export
```

---

## 4. Import Statements

### 4.1 Basic Imports

```groklang
use std::io::File;           // Single item
use std::io::File, Reader;   // Multiple items
use std::io::*;              // Everything (wildcard)
```

### 4.2 Renaming

```groklang
use std::io::File as StdFile;  // Avoid name conflicts
use std::io::*;

// Later
let f = StdFile::open("file.txt");
```

### 4.3 Nested Imports

```groklang
use std::io::{File, Reader, self};  // Also import io itself

// Equivalent to:
use std::io::File;
use std::io::Reader;
use std::io;
```

### 4.4 Glob Imports

```groklang
use std::io::*;  // Import all public items from std::io

// Avoid in library code (pollutes namespace)
// Acceptable in binary code
```

### 4.5 External Crates

```groklang
use serde::Serialize;

// In grok.toml:
[dependencies]
serde = "1.0"
```

---

## 5. Package Configuration (grok.toml)

### 5.1 Manifest Format

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2026"
authors = ["Author Name <email>"]
license = "MIT"
description = "A brief description"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
quickcheck = "1.0"

[build-dependencies]
cc = "1"

[lib]
name = "my_project"
path = "src/lib.grok"

[[bin]]
name = "my_app"
path = "src/main.grok"

[features]
default = ["feature1"]
feature1 = []
feature2 = ["serde"]  # Enables serde when feature2 enabled
```

### 5.2 Workspace Configuration

```toml
# workspace.toml
[workspace]
members = [
    "crate1",
    "crate2",
    "tools",
]

[workspace.dependencies]
# Shared dependencies across workspace
serde = "1.0"
tokio = "1"
```

---

## 6. Naming Conventions

### 6.1 Crate Names

- **Snake case**: `my_awesome_library`
- **Meaningful**: `tokenizer`, `json_parser`, `async_runtime`
- **Unique**: Avoid conflicts on registry

### 6.2 Module Names

```groklang
// Snake case
mod http_server { }
mod json_parser { }

// Not:
// mod HTTPServer { }    ← PascalCase (not convention)
// mod json-parser { }   ← Kebab-case (not allowed)
```

### 6.3 Item Names

```groklang
// Type names: PascalCase
struct MyStruct { }
enum MyEnum { }
trait MyTrait { }

// Function/method names: snake_case
fn my_function() { }
impl MyStruct {
    fn my_method(&self) { }
}

// Constant names: UPPER_SNAKE_CASE
const MAX_CONNECTIONS: i32 = 100;
static GLOBAL_STATE: i32 = 0;

// Type variables: Single uppercase letter (or PascalCase)
fn generic<T>(x: T) { }
fn named<MyType : Clone>(x: MyType) { }
```

---

## 7. Prelude

### 7.1 Standard Prelude

Automatically imported (no `use` needed):

```groklang
// Implicitly available everywhere:
use grok::prelude::*;

// Which includes:
pub use crate::option::Option;
pub use crate::result::Result;
pub use crate::vec::Vec;
pub use crate::string::String;
pub use crate::panic;
pub use crate::println;
// ... and other common items
```

### 7.2 Custom Preludes

```groklang
// In library root (lib.grok):
pub mod prelude {
    pub use crate::core::*;
    pub use crate::traits::*;
}

// Consumers can:
use my_library::prelude::*;
```

---

## 8. Crate Types

### 8.1 Library Crate

```toml
[lib]
name = "my_library"
path = "src/lib.grok"
```

Entry point: `src/lib.grok`

```groklang
// src/lib.grok
pub mod public_module;
mod private_module;

pub fn public_function() { }
```

### 8.2 Binary Crate

```toml
[[bin]]
name = "my_app"
path = "src/main.grok"
```

Entry point: `src/main.grok`

```groklang
// src/main.grok
fn main() {
    println!("Hello, World!");
}
```

### 8.3 Test Crate

```
tests/
└── integration_test.grok
```

```groklang
// tests/integration_test.grok
use my_library::*;

#[test]
fn test_library() {
    // Test code
}
```

---

## 9. Version Management

### 9.1 Semantic Versioning

Format: `MAJOR.MINOR.PATCH`

```
1.0.0   -- Initial release
1.1.0   -- New features (backward compatible)
1.1.1   -- Bug fixes
2.0.0   -- Breaking changes
```

### 9.2 Dependency Specifications

```toml
serde = "1.0"              # Exact: 1.0.x
serde = "1"                # Caret: 1.y.z (< 2.0.0)
serde = "1.2"              # Caret: 1.2.z (< 2.0.0)
serde = "1.2.3"            # Exact: 1.2.3 only
serde = "~1.2.3"           # Tilde: >= 1.2.3, < 1.3.0
serde = ">=1.2, <2"        # Ranges
```

---

## 10. Conditional Compilation

### 10.1 Feature Gates

```groklang
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
pub struct Data {
    value: i32,
}

#[cfg(not(feature = "serde"))]
pub struct Data {
    value: i32,
}
```

### 10.2 Platform-specific Code

```groklang
#[cfg(target_os = "windows")]
fn platform_specific() { }

#[cfg(all(target_os = "unix", not(target_os = "macos")))]
fn linux_only() { }

#[cfg(target_pointer_width = "64")]
fn only_64bit() { }
```

### 10.3 Debug vs Release

```groklang
#[cfg(debug_assertions)]
fn debugging_function() { }

#[cfg(not(debug_assertions))]
fn production_function() { }
```

---

## 11. Module System Examples

### 11.1 Web Framework Example

```
web_framework/
├── src/
│   ├── lib.grok
│   ├── http/
│   │   ├── request.grok
│   │   ├── response.grok
│   │   └── status.grok
│   ├── routing/
│   │   ├── router.grok
│   │   └── handler.grok
│   └── middleware/
│       ├── auth.grok
│       └── cors.grok
└── grok.toml

// src/lib.grok:
mod http;
mod routing;
mod middleware;

pub use http::{Request, Response, Status};
pub use routing::Router;
pub use middleware::{Middleware, Auth};

// src/http/mod.grok:
mod request;
mod response;
pub mod status;

pub use request::Request;
pub use response::Response;
```

### 11.2 Game Engine Example

```
game_engine/
├── src/
│   ├── lib.grok
│   ├── core/
│   │   ├── entity.grok
│   │   ├── component.grok
│   │   └── system.grok
│   ├── graphics/
│   │   ├── renderer.grok
│   │   └── shader.grok
│   └── physics/
│       ├── rigid_body.grok
│       └── collision.grok

// src/lib.grok:
pub mod core;
pub mod graphics;
pub mod physics;

// Re-export commonly used items
pub use core::{Entity, Component, System};
pub use graphics::Renderer;
pub use physics::RigidBody;
```

---

## 12. Circular Dependencies

### 12.1 Detecting Cycles

Compiler detects and rejects circular dependencies:

```groklang
// module_a.grok
use crate::module_b;  // ERROR if module_b imports module_a

// module_b.grok
use crate::module_a;  // Circular dependency!
```

### 12.2 Resolution via Traits

Use traits to break cycles:

```groklang
// core/mod.grok
pub trait Handler {
    fn process(&self, data: Data);
}

// http/mod.grok
use crate::core::Handler;

pub struct HttpHandler;
impl Handler for HttpHandler { }

// middleware/mod.grok
use crate::core::Handler;

pub fn wrap<H: Handler>(handler: H) { }
```

---

## 13. Standard Library Organization

### 13.1 grok::core

Core language functionality:

```groklang
use grok::core::{
    // Primitives
    i32, f64, bool, char, str,

    // Collections
    Vec, String, HashMap, HashSet,

    // Control
    Option, Result,

    // Utilities
    Clone, Debug, Default,
};
```

### 13.2 grok::std

Standard library modules:

```groklang
use grok::std::{
    io::{File, Read, Write},
    fs, net, time, path,
    sync::{Mutex, RwLock, Arc},
    thread,
    process,
};
```

---

## 14. Validation Criteria

- [ ] Module paths resolve correctly
- [ ] Visibility rules enforced
- [ ] Imports work (direct, wildcard, nested)
- [ ] Circular dependencies detected
- [ ] Crate.toml parsed correctly
- [ ] Dependencies resolved
- [ ] Conditional compilation works
- [ ] Re-exports work
- [ ] Prelude available
- [ ] Name resolution correct
- [ ] Error messages clear

---

## 15. Related Documents

- [01-Architecture-Overview.md](01-Architecture-Overview.md) - System architecture
- [03-Syntax-Grammar.md](03-Syntax-Grammar.md) - Grammar for use statements
- [07-Standard-Library-API.md](07-Standard-Library-API.md) - Standard library organization
