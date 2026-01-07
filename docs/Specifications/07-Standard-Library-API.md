# GrokLang Standard Library API Specification

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Scope**: Core library modules and APIs

---

## 1. Standard Library Organization

```
grok::
├── core          -- Core types and traits
├── std           -- Standard library
├── collections   -- Data structures
├── io            -- Input/Output
├── fs            -- File system
├── net           -- Networking
├── thread        -- Threading
├── sync          -- Synchronization
├── time          -- Time utilities
├── process      -- Process management
├── path          -- Path manipulation
├── env           -- Environment variables
├── sys           -- System information
├── ffi           -- Foreign function interface
└── ai            -- AI integration
```

---

## 2. Core Module (grok::core)

### 2.1 Primitive Types

```groklang
// Numeric types
i8, i16, i32, i64, i128, isize
u8, u16, u32, u64, u128, usize
f32, f64

// Other primitives
bool    -- Boolean true/false
char    -- Unicode scalar (32-bit)
str     -- String slice
()      -- Unit type

// Traits for all primitives
trait Clone {
    fn clone(self) -> Self;
}

trait Copy {
    // Marker trait, auto-implemented for primitives
}

trait Eq {
    fn eq(&self, other: &Self) -> bool;
}

trait Ord : Eq {
    fn cmp(&self, other: &Self) -> Ordering;
}
```

### 2.2 Option and Result

```groklang
pub enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    pub fn is_some(self) -> bool;
    pub fn is_none(self) -> bool;
    pub fn unwrap(self) -> T;
    pub fn unwrap_or(self, default: T) -> T;
    pub fn map<U>(self, f: fn(T) -> U) -> Option<U>;
    pub fn and_then<U>(self, f: fn(T) -> Option<U>) -> Option<U>;
}

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn is_ok(self) -> bool;
    pub fn is_err(self) -> bool;
    pub fn unwrap(self) -> T;
    pub fn map<U>(self, f: fn(T) -> U) -> Result<U, E>;
    pub fn map_err<F>(self, f: fn(E) -> F) -> Result<T, F>;
}
```

### 2.3 Tuples

```groklang
// Tuple operations
let pair: (i32, str) = (42, "hello");
let (x, y) = pair;              // Destructuring
let x = pair.0;                 // Field access
let y = pair.1;

// Traits
impl<T: Clone, U: Clone> Clone for (T, U) { }
impl<T: Eq, U: Eq> Eq for (T, U) { }
```

---

## 3. Collections Module (grok::collections)

### 3.1 Vec (Vector)

```groklang
pub struct Vec<T> {
    data: *mut T,
    capacity: usize,
    len: usize,
}

impl<T> Vec<T> {
    // Construction
    pub fn new() -> Vec<T>;
    pub fn with_capacity(capacity: usize) -> Vec<T>;
    pub fn from_slice(slice: &[T]) -> Vec<T>;

    // Modification
    pub fn push(mut self, value: T) -> ();
    pub fn pop(mut self) -> Option<T>;
    pub fn insert(mut self, index: usize, value: T) -> ();
    pub fn remove(mut self, index: usize) -> T;
    pub fn clear(mut self) -> ();

    // Query
    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;
    pub fn capacity(self) -> usize;
    pub fn get(self, index: usize) -> Option<&T>;
    pub fn get_mut(mut self, index: usize) -> Option<&mut T>;

    // Iteration
    pub fn iter(self) -> Iterator<T>;
    pub fn iter_mut(mut self) -> Iterator<&mut T>;
    pub fn into_iter(self) -> Iterator<T>;
}

// Macro
macro_rules! vec {
    ($($item:expr),* $(,)?) => { /* ... */ };
    ($item:expr; $count:expr) => { /* ... */ };
}

// Usage
let v = vec![1, 2, 3];
let v = vec![0; 10];
```

### 3.2 String

```groklang
pub struct String {
    data: Vec<u8>,
}

impl String {
    pub fn new() -> String;
    pub fn from_str(s: &str) -> String;

    pub fn push(mut self, c: char) -> ();
    pub fn push_str(mut self, s: &str) -> ();
    pub fn pop(mut self) -> Option<char>;

    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;
    pub fn chars(self) -> Iterator<char>;
    pub fn as_str(self) -> &str;
    pub fn as_bytes(self) -> &[u8];

    pub fn split(self, separator: char) -> Iterator<&str>;
    pub fn trim(self) -> &str;
    pub fn to_uppercase(self) -> String;
    pub fn to_lowercase(self) -> String;
}

// String interpolation (macro)
macro_rules! format {
    ($fmt:expr $(, $arg:expr)*) => { /* ... */ };
}

// Usage
let s = String::from_str("hello");
let s = format!("answer: {}", 42);
```

### 3.3 HashMap

```groklang
pub struct HashMap<K, V> {
    // Implementation details
}

impl<K: Eq + Hash, V> HashMap<K, V> {
    pub fn new() -> HashMap<K, V>;
    pub fn with_capacity(capacity: usize) -> HashMap<K, V>;

    pub fn insert(mut self, key: K, value: V) -> Option<V>;
    pub fn remove(mut self, key: &K) -> Option<V>;
    pub fn clear(mut self) -> ();

    pub fn get(self, key: &K) -> Option<&V>;
    pub fn get_mut(mut self, key: &K) -> Option<&mut V>;
    pub fn contains_key(self, key: &K) -> bool;

    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;

    pub fn keys(self) -> Iterator<&K>;
    pub fn values(self) -> Iterator<&V>;
    pub fn iter(self) -> Iterator<(&K, &V)>;
}

// Usage
let map = HashMap::new();
map.insert("key", 42);
```

### 3.4 HashSet

```groklang
pub struct HashSet<T> {
    data: HashMap<T, ()>,
}

impl<T: Eq + Hash> HashSet<T> {
    pub fn new() -> HashSet<T>;
    pub fn insert(mut self, value: T) -> bool;  // Returns true if new
    pub fn remove(mut self, value: &T) -> bool;
    pub fn contains(self, value: &T) -> bool;

    pub fn len(self) -> usize;
    pub fn iter(self) -> Iterator<&T>;
}
```

---

## 4. IO Module (grok::io)

### 4.1 Traits

```groklang
pub trait Read {
    fn read(mut self, buf: &mut [u8]) -> Result<usize, IoError>;

    fn read_to_end(mut self, buf: &mut Vec<u8>) -> Result<usize, IoError> {
        // Default implementation
    }
}

pub trait Write {
    fn write(mut self, buf: &[u8]) -> Result<usize, IoError>;
    fn flush(mut self) -> Result<(), IoError>;

    fn write_all(mut self, buf: &[u8]) -> Result<(), IoError> {
        // Default implementation
    }
}
```

### 4.2 Standard Streams

```groklang
// Standard input
pub fn stdin() -> Stdin;

pub struct Stdin { /* ... */ }
impl Read for Stdin { }

// Standard output
pub fn stdout() -> Stdout;

pub struct Stdout { /* ... */ }
impl Write for Stdout { }

// Standard error
pub fn stderr() -> Stderr;

pub struct Stderr { /* ... */ }
impl Write for Stderr { }

// Macros
macro_rules! print {
    ($fmt:expr $(, $arg:expr)*) => { /* ... */ };
}

macro_rules! println {
    ($fmt:expr $(, $arg:expr)*) => { /* ... */ };
}

macro_rules! eprint {
    ($fmt:expr $(, $arg:expr)*) => { /* ... */ };
}

macro_rules! eprintln {
    ($fmt:expr $(, $arg:expr)*) => { /* ... */ };
}

// Usage
println!("Hello {}", "world");
eprintln!("Error: {}", error);
```

### 4.3 Error Type

```groklang
pub struct IoError {
    kind: IoErrorKind,
    message: String,
}

pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    InvalidData,
    TimedOut,
    Other,
}
```

---

## 5. File System Module (grok::fs)

### 5.1 File Operations

```groklang
pub struct File {
    handle: i32,
}

impl File {
    pub fn open(path: &str) -> Result<File, IoError>;
    pub fn create(path: &str) -> Result<File, IoError>;
    pub fn read(mut self) -> Result<Vec<u8>, IoError>;
}

impl Read for File { }
impl Write for File { }
impl Drop for File {
    fn drop(mut self) { /* Close file */ }
}

// Usage
let f = File::open("data.txt")?;
let contents = f.read()?;
```

### 5.2 Directory Operations

```groklang
pub fn read_dir(path: &str) -> Result<Iterator<DirEntry>, IoError>;
pub fn create_dir(path: &str) -> Result<(), IoError>;
pub fn create_dir_all(path: &str) -> Result<(), IoError>;
pub fn remove_dir(path: &str) -> Result<(), IoError>;
pub fn remove_file(path: &str) -> Result<(), IoError>;

pub struct DirEntry {
    path: String,
    metadata: Metadata,
}

pub struct Metadata {
    size: u64,
    modified: SystemTime,
    is_file: bool,
    is_dir: bool,
}

// Usage
for entry in read_dir(".")? {
    println!("{}", entry.path);
}
```

---

## 6. Threading Module (grok::thread)

### 6.1 Thread Spawning

```groklang
pub fn spawn<F>(f: F) -> JoinHandle<F::Output>
where F: Fn() -> T + Send + 'static, T: Send + 'static;

pub struct JoinHandle<T> { /* ... */ }

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, ()>;
}

// Macro helper
macro_rules! spawn {
    ($body:block) => { spawn(fn() $body) };
}

// Usage
let handle = spawn {
    println!("running in thread");
    42
};
let result = handle.join();
```

### 6.2 Thread Local Storage

```groklang
pub struct ThreadLocal<T> {
    // Implementation
}

impl<T: 'static> ThreadLocal<T> {
    pub fn new(init: fn() -> T) -> ThreadLocal<T>;
    pub fn with<F>(self, f: F) -> F::Output
    where F: Fn(&T) -> R;
}

// Macro
macro_rules! thread_local {
    ($init:expr) => { ThreadLocal::new(fn() $init) };
}
```

---

## 7. Synchronization Module (grok::sync)

### 7.1 Mutex

```groklang
pub struct Mutex<T> { /* ... */ }

impl<T> Mutex<T> {
    pub fn new(value: T) -> Mutex<T>;
    pub fn lock(mut self) -> MutexGuard<T>;
    pub fn try_lock(mut self) -> Option<MutexGuard<T>>;
}

pub struct MutexGuard<'a, T> { /* ... */ }

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(self) -> &T;
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(mut self) -> &mut T;
}

// Usage
let m = Mutex::new(42);
{
    let guard = m.lock();
    let value = *guard;  // Dereference to access
}  // Guard dropped, lock released
```

### 7.2 RwLock

```groklang
pub struct RwLock<T> { /* ... */ }

impl<T> RwLock<T> {
    pub fn read(self) -> RwLockReadGuard<T>;
    pub fn write(mut self) -> RwLockWriteGuard<T>;
}

// Usage
let lock = RwLock::new(vec![1, 2, 3]);
let read_guard = lock.read();
let item = read_guard[0];

let write_guard = lock.write();
write_guard[0] = 42;
```

### 7.3 Channel

```groklang
pub fn channel<T>() -> (Sender<T>, Receiver<T>);

pub struct Sender<T> { /* ... */ }

impl<T: Send + 'static> Sender<T> {
    pub fn send(self, value: T) -> Result<(), SendError<T>>;
}

pub struct Receiver<T> { /* ... */ }

impl<T: Send + 'static> Receiver<T> {
    pub fn recv(mut self) -> Result<T, RecvError>;
    pub fn try_recv(mut self) -> Result<T, TryRecvError>;
}

// Usage
let (tx, rx) = channel();
spawn { tx.send(42); };
let value = rx.recv();
```

---

## 8. Time Module (grok::time)

### 8.1 Duration

```groklang
pub struct Duration {
    secs: u64,
    nanos: u32,
}

impl Duration {
    pub fn new(secs: u64, nanos: u32) -> Duration;
    pub fn from_secs(secs: u64) -> Duration;
    pub fn from_millis(millis: u64) -> Duration;
    pub fn as_secs(self) -> u64;
    pub fn as_millis(self) -> u128;
}
```

### 8.2 SystemTime

```groklang
pub struct SystemTime { /* ... */ }

impl SystemTime {
    pub fn now() -> SystemTime;
    pub fn duration_since(self, earlier: SystemTime) -> Result<Duration, SystemTimeError>;
    pub fn elapsed(self) -> Result<Duration, SystemTimeError>;
}

// Usage
let start = SystemTime::now();
do_work();
let elapsed = start.elapsed()?;
println!("took: {:?}", elapsed);
```

---

## 9. Iterator Trait

### 9.1 Iterator Trait

```groklang
pub trait Iterator {
    type Item;

    fn next(mut self) -> Option<Self::Item>;

    // Default implementations
    fn map<U, F>(self, f: F) -> MapIterator<Self, F>
    where F: Fn(Self::Item) -> U;

    fn filter<F>(self, predicate: F) -> FilterIterator<Self, F>
    where F: Fn(&Self::Item) -> bool;

    fn fold<U, F>(mut self, init: U, f: F) -> U
    where F: Fn(U, Self::Item) -> U;

    fn collect<C>(self) -> C
    where C: FromIterator<Self::Item>;

    fn count(mut self) -> usize;
    fn sum(mut self) -> Self::Item;
    fn max(mut self) -> Option<Self::Item>;
    fn min(mut self) -> Option<Self::Item>;
}

// Usage
let v = vec![1, 2, 3, 4, 5];
let doubled: Vec<i32> = v.iter()
    .map(|x| x * 2)
    .filter(|x| x > 4)
    .collect();
```

---

## 10. String Formatting

### 10.1 Format Trait

```groklang
pub trait Display {
    fn fmt(self, f: &mut Formatter) -> Result<(), FmtError>;
}

pub trait Debug {
    fn fmt(self, f: &mut Formatter) -> Result<(), FmtError>;
}

// Derive macros
#[derive(Display)]
#[derive(Debug)]

// Usage
impl Display for MyStruct {
    fn fmt(self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "MyStruct {{ ... }}")
    }
}

let s = format!("{}", my_struct);
let s = format!("{:?}", my_struct);
```

---

## 11. Foreign Function Interface (grok::ffi)

### 11.1 FFI Types

```groklang
pub struct CStr { /* ... */ }
pub struct CString { /* ... */ }

// C-compatible types
pub type c_char = i8;
pub type c_int = i32;
pub type c_long = i64;
pub type c_float = f32;
pub type c_double = f64;
```

### 11.2 External Functions

```groklang
#[ai_translate]
extern "C" {
    pub fn strlen(s: *const c_char) -> c_int;
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void) -> ();
}

#[ai_translate]
extern "py" {
    pub fn numpy_sum(arr: list) -> f64;
}
```

---

## 12. AI Module (grok::ai)

### 12.1 AI Service Interface

```groklang
pub trait AiService {
    fn call(request: AiRequest) -> Result<AiResponse, AiError>;
}

pub struct AiRequest {
    operation: String,
    input: String,
    parameters: Map<String, String>,
    timeout: Duration,
}

pub struct AiResponse {
    output: String,
    explanation: String,
    metrics: Map<String, f64>,
}

pub enum AiError {
    Timeout,
    ServiceUnavailable,
    InvalidInput,
    TransformationFailed(String),
}

// Configuration
pub fn configure(backend: AiBackend) -> ();

pub enum AiBackend {
    Local { url: String, model: String },
    OpenAi { api_key: String, model: String },
    Offline,
}
```

---

## 13. Standard Traits and Macros

### 13.1 Derivable Traits

```groklang
#[derive(Clone)]     // Implement Clone
#[derive(Copy)]      // Implement Copy (marker)
#[derive(Eq)]        // Implement Eq (equality)
#[derive(Ord)]       // Implement Ord (ordering)
#[derive(Hash)]      // Implement Hash
#[derive(Default)]   // Implement Default
#[derive(Debug)]     // Implement Debug
#[derive(Display)]   // Implement Display
```

### 13.2 Useful Macros

```groklang
// Assertions
assert!(condition);
assert_eq!(left, right);
assert_ne!(left, right);

// Panic
panic!("message");
unreachable!();

// Debugging
todo!();        // Mark unimplemented
dbg!(expr);     // Debug print

// Collections
vec![...]
map!{...}
set!{...}

// String interpolation
format!("...")
println!("...")
```

---

## 14. Validation Criteria

- [ ] All primitive types work
- [ ] Vec operations functional
- [ ] String operations functional
- [ ] HashMap/HashSet work
- [ ] File I/O works
- [ ] Threading works
- [ ] Synchronization primitives work
- [ ] Channels work
- [ ] Iterator trait works
- [ ] Display/Debug traits work
- [ ] FFI works
- [ ] Macros expand correctly
- [ ] Error handling propagates
- [ ] All derives work

---

## 15. Related Documents

- [01-Architecture-Overview.md](01-Architecture-Overview.md) - System architecture
- [06-Module-System.md](06-Module-System.md) - Module organization
- [04-Runtime-Memory-Model.md](04-Runtime-Memory-Model.md) - Memory management
