# GrokLang Runtime and Memory Model Specification

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Scope**: Memory management, ownership, concurrency runtime

---

## 1. Memory Model Overview

GrokLang uses a **hybrid memory model**:

1. **Stack allocation** (default): Fast, automatic cleanup
2. **Heap allocation** (explicit): Via `Box<T>`, `Vec<T>`, etc.
3. **Borrow checking** (compile-time): Prevents memory unsafety
4. **Reference counting** (runtime): Shared ownership via `Rc<T>` and `Arc<T>`
5. **AI-managed GC** (optional): Supplementary garbage collection

---

## 2. Ownership System

### 2.1 Ownership Rules

Every value in GrokLang has exactly one owner at any moment.

**Rule 1**: When the owner goes out of scope, the value is dropped

```groklang
{
    let x = String::new("hello");
    // x is owner of string
} // x goes out of scope, string is dropped
```

**Rule 2**: Ownership can be transferred (moved)

```groklang
let s1 = String::new("hello");
let s2 = s1;  // Ownership of string transfers to s2
// s1 is no longer valid
println!(s1);  // ERROR: s1 has been moved
```

**Rule 3**: Ownership can be borrowed (temporarily loaned)

```groklang
let s1 = String::new("hello");
let s2 = &s1;    // s2 borrows s1 immutably
let s3 = &s1;    // Multiple immutable borrows OK
// s1 still owns the string
println!(s1);    // OK
```

### 2.2 Copy vs Move Semantics

**Copy types**: Trivial to copy (primitives)

- `i32`, `f64`, `bool`, `char`
- Tuples of copy types
- Copying is implicit, ownership doesn't transfer

**Move types**: Expensive to copy (heap-allocated)

- `String`, `Vec<T>`, custom structs
- Ownership transfers on assignment
- Can implement `Copy` trait (marker only for copy types)

```groklang
// Copy semantics
let x = 42;
let y = x;  // x is copied, both x and y valid
println!("{}", x);  // OK

// Move semantics
let s1 = vec![1, 2, 3];
let s2 = s1;  // s1 moved to s2
println!("{}", s1);  // ERROR: s1 moved

// Explicit copy
let s3 = s1.clone();  // s1 cloned, both valid
println!("{}", s1);   // OK
```

---

## 3. Borrowing and References

### 3.1 Immutable References (`&T`)

An immutable reference allows reading but not modification:

```groklang
fn read_string(s: &String) -> usize {
    s.len()  // Can read
}

let s = String::new("hello");
let r1 = &s;    // Immutable borrow
let r2 = &s;    // Multiple immutable borrows OK
println!("{} {}", r1, r2);  // OK
```

**Immutable borrow rules**:

- Multiple immutable borrows allowed simultaneously
- Borrowed value cannot be modified
- Owner cannot modify while borrowed

### 3.2 Mutable References (`&mut T`)

A mutable reference allows reading and modification:

```groklang
fn append_char(s: &mut String, c: char) -> () {
    s.push(c);  // Can modify
}

let mut s = String::new("hello");
let r = &mut s;       // Mutable borrow
r.push('!');          // OK
// s.push('!');       // ERROR: s is borrowed mutably
println!("{}", r);    // OK
```

**Mutable borrow rules**:

- Only one mutable borrow allowed at a time
- No other borrows (immutable or mutable) allowed simultaneously
- Borrowed value cannot be directly used while borrowed

### 3.3 Borrow Checker Algorithm

The compiler uses a flow analysis algorithm to verify safety:

1. **Borrow tracking**: Track all borrows and their lifetimes
2. **Conflict detection**: Detect overlapping borrows
3. **Lifetime verification**: Ensure borrows don't outlive referent
4. **Liveness analysis**: Verify value still alive when borrowed

```groklang
fn example() -> () {
    let mut x = 5;
    let r1 = &x;       // Immutable borrow (r1)
    let r2 = &x;       // Another immutable borrow (r2)
    // x = 10;         // ERROR: x mutably borrowed later
    println!("{}", r1 + r2);  // r1, r2 used, borrow ends here
    x = 10;            // OK: no active borrows
}
```

---

## 4. Lifetime Tracking

### 4.1 Lifetime Parameters

Lifetimes are explicitly tracked for references:

```groklang
struct Ref<'a, T> {
    ptr: &'a T,
}

fn first<'a>(items: &'a [T]) -> &'a T {
    &items[0]
}

impl<'a, T> Ref<'a, T> {
    fn get(self) -> &'a T {
        self.ptr
    }
}
```

**Notation**:

- `'a`, `'b`, `'c`: Lifetime variables
- `'static`: Special lifetime for program duration
- `&'a T`: Reference to T valid for lifetime 'a

### 4.2 Lifetime Constraints

```groklang
// Input lifetime
fn first(items: &[T]) -> &T {
    // Input lifetime automatically propagated to output
}
// Equivalent to: fn first<'a>(items: &'a [T]) -> &'a T

// Multiple lifetimes
fn join<'a, 'b>(x: &'a T, y: &'b T) -> &'a T {
    // Output lifetime is 'a (shorter of the two)
    x
}

// Self lifetime
impl String {
    fn as_str(&self) -> &str {
        // &self's lifetime automatically used for return
    }
}
```

### 4.3 Lifetime Elision Rules

Most lifetime annotations are elided (inferred):

**Rule 1**: Each input reference gets its own lifetime

```groklang
fn first(x: &T)                    // x has lifetime 'a
fn join(x: &T, y: &U)              // x: 'a, y: 'b
```

**Rule 2**: If single input lifetime, output gets that lifetime

```groklang
fn get_str(x: &String) -> &str     // Output: 'a (from x)
fn clone(x: &T) -> T               // If T implements Clone
```

**Rule 3**: If `&self`, output gets self's lifetime

```groklang
impl T {
    fn method(&self) -> &U {       // Output: self's lifetime
        ...
    }
}
```

---

## 5. Memory Layout and Representation

### 5.1 Stack Layout

Values are laid out on stack in order of declaration:

```groklang
fn example() -> () {
    let a: i32 = 1;            // Stack[0] = 1 (4 bytes)
    let b: f64 = 2.0;          // Stack[4] = 2.0 (8 bytes)
    let c: bool = true;        // Stack[12] = 1 (1 byte)
}
// Stack freed on function return
```

**Stack properties**:

- Very fast allocation/deallocation
- No fragmentation
- Limited size (typically 2-8MB)
- Automatic cleanup (RAII pattern)

### 5.2 Heap Layout

Heap allocations managed explicitly:

```groklang
let v = vec![1, 2, 3];        // Allocates on heap
                              // v contains: ptr, capacity, length

let s = String::new("hello"); // Allocates on heap
                              // s contains: ptr, capacity, length
```

**Heap properties**:

- Unlimited size
- Slower allocation/deallocation
- Manual cleanup (via Drop trait)
- Fragmentation possible

### 5.3 Type Sizes and Alignment

Sizes determined at compile time:

```groklang
// Primitives
size_of::<i32>() == 4
size_of::<f64>() == 8
size_of::<bool>() == 1

// Structures
struct Point {
    x: f64,      // Offset 0, 8 bytes
    y: f64,      // Offset 8, 8 bytes
}
size_of::<Point>() == 16

// Enums (discriminant + largest variant)
enum Option<T> {
    Some(T),     // 1 byte (discriminant) + size_of::<T>()
    None,
}

// Generics monomorphized
Vec<i32>  // Pointer-sized: 3 words (ptr, capacity, length)
Vec<f64>  // Same size: 3 words (element size irrelevant)
```

---

## 6. Drop and Resource Management (RAII)

### 6.1 Drop Trait

Every type can implement `Drop` to clean up resources:

```groklang
trait Drop {
    fn drop(mut self) -> ();
}

struct File {
    handle: i32,
}

impl Drop for File {
    fn drop(mut self) -> () {
        close(self.handle);
    }
}

fn example() -> () {
    let f = File { handle: 42 };
    // ...
} // f.drop() called automatically
```

### 6.2 Resource Acquisition Is Initialization (RAII)

Resources tied to object lifetime:

```groklang
struct Mutex<T> {
    value: T,
    lock: i32,
}

impl<T> Mutex<T> {
    fn lock(mut self) -> Guard<T> {
        acquire_lock(&mut self.lock);
        Guard { mutex: &mut self, .. }
    }
}

impl<T> Drop for Guard<T> {
    fn drop(mut self) -> () {
        release_lock(&mut self.mutex.lock);
    }
}

fn example() -> () {
    let m = Mutex { value: 42, lock: 0 };
    {
        let guard = m.lock();
        // Critical section
    } // guard.drop() called, lock released
}
```

---

## 7. Shared Ownership

### 7.1 Reference Counting (`Rc<T>`)

Single-threaded shared ownership:

```groklang
struct Rc<T> {
    ptr: *const RcBox<T>,
}

struct RcBox<T> {
    strong_count: i32,
    weak_count: i32,
    value: T,
}

impl<T> Rc<T> {
    fn new(value: T) -> Rc<T> {
        let box = RcBox { strong_count: 1, weak_count: 0, value };
        Rc { ptr: &box }
    }

    fn clone(self) -> Rc<T> {
        (*self.ptr).strong_count += 1;
        Rc { ptr: self.ptr }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(mut self) -> () {
        (*self.ptr).strong_count -= 1;
        if (*self.ptr).strong_count == 0 {
            // Deallocate value
        }
    }
}
```

**Usage**:

```groklang
let x = Rc::new(42);
let y = x.clone();
let z = x.clone();
// Three owners of 42
```

### 7.2 Atomic Reference Counting (`Arc<T>`)

Thread-safe shared ownership:

```groklang
struct Arc<T> {
    ptr: *const ArcBox<T>,
}

struct ArcBox<T> {
    strong_count: AtomicI32,
    weak_count: AtomicI32,
    value: T,
}

// Same interface as Rc but with atomic operations
// Safe to share across threads
```

**Usage**:

```groklang
let x = Arc::new(42);
spawn {
    let y = x.clone();  // Thread-safe clone
    println!("{}", y);
};
```

---

## 8. Concurrency Runtime

### 8.1 Lightweight Threads

Threads mapped to OS threads or green threads:

```groklang
fn spawn(f: fn() -> ()) -> JoinHandle<()> {
    // Platform-dependent implementation
    // Returns handle to wait for completion
}

// Usage
let handle = spawn {
    println!("running in thread");
};
handle.join();  // Wait for completion
```

**Thread safety**:

- Data must be `Send`: Safe to send across threads
- Data must be `Sync`: Safe to share via `&T` across threads
- Compiler enforces via type system

```groklang
trait Send {
    // Auto-trait, implemented for most types
    // Types safe to send across threads
}

trait Sync {
    // Auto-trait
    // Types safe to share via &T across threads
}

// Counterexample: Rc is !Send and !Sync
// But Arc is Send + Sync
```

### 8.2 Message Passing

Channels for thread communication:

```groklang
fn channel<T>() -> (Sender<T>, Receiver<T>) {
    // Creates a message queue
}

struct Sender<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
}

struct Receiver<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
}

impl<T : Send + 'static> Sender<T> {
    fn send(mut self, value: T) -> () {
        self.queue.lock().push_back(value);
    }
}

impl<T : Send + 'static> Receiver<T> {
    fn recv(mut self) -> Option<T> {
        self.queue.lock().pop_front()
    }
}

// Usage
let (tx, rx) = channel::<i32>();
spawn {
    tx.send(42);
};
let value = rx.recv();  // Blocks until value received
```

### 8.3 Synchronization Primitives

#### Mutex (Mutual Exclusion)

```groklang
struct Mutex<T> {
    lock: AtomicBool,
    value: T,
}

impl<T> Mutex<T> {
    fn lock(mut self) -> MutexGuard<T> {
        // Spin until lock acquired
        while self.lock.compare_and_swap(false, true) != false {
            // Spin or yield
        }
        MutexGuard { mutex: &mut self }
    }
}

impl<T> Drop for MutexGuard<T> {
    fn drop(mut self) -> () {
        self.mutex.lock.store(false);
    }
}

// Usage
let m = Mutex::new(42);
let guard = m.lock();
let value = *guard;  // Can read
// guard.drop() called, lock released
```

#### RwLock (Reader-Writer Lock)

```groklang
struct RwLock<T> {
    readers: i32,
    writers: i32,
    value: T,
}

impl<T> RwLock<T> {
    fn read(mut self) -> RwLockReadGuard<T> {
        while self.writers > 0 { /* wait */ }
        self.readers += 1;
        RwLockReadGuard { lock: &mut self }
    }

    fn write(mut self) -> RwLockWriteGuard<T> {
        while self.readers > 0 || self.writers > 0 { /* wait */ }
        self.writers += 1;
        RwLockWriteGuard { lock: &mut self }
    }
}
```

#### Semaphore

```groklang
struct Semaphore {
    count: AtomicI32,
}

impl Semaphore {
    fn wait(self) -> () {
        loop {
            let current = self.count.load();
            if current > 0 && self.count.compare_and_swap(current, current - 1) == current {
                break;
            }
        }
    }

    fn post(self) -> () {
        self.count.fetch_add(1);
    }
}
```

---

## 9. Actor Model (Optional)

### 9.1 Actor Definitions

Isolated concurrent entities communicating via messages:

```groklang
actor MyActor {
    state: i32,

    fn behave() -> () {
        receive(message) {
            state += message;
        }
    }
}

// Usage
let actor = MyActor { state: 0 };
actor.send(42);     // Asynchronous message send
```

### 9.2 Actor Scheduling

Actor framework handles scheduling:

- Each actor has message queue
- Mailbox processed sequentially
- Isolation ensures no data races
- Deadlock detection monitors for cycles

---

## 10. AI-Managed Garbage Collection

### 10.1 Optional GC Pool

For scenarios where borrow checker too restrictive:

```groklang
let x = ai::alloc();         // Allocate in AI-managed pool
let y = ai::alloc();
// AI tracks usage and collects when safe

// Or via attribute
#[ai_managed]
fn process() -> () {
    // All allocations in this function AI-managed
}
```

### 10.2 AI Collection Strategy

AI determines optimal collection strategy:

- **Generational GC**: Most objects die young
- **Incremental GC**: Low pause times
- **Concurrent GC**: Run alongside program
- **Mark-and-sweep**: For object graphs

### 10.3 Safety Guarantee

AI-GC is **sound** but not **complete**:

- No undefined behavior
- Memory reclaimed eventually
- May be conservative (keep alive objects longer)
- Borrow checker still enforced

---

## 11. Runtime Initialization and Cleanup

### 11.1 Module Initialization

```groklang
mod math {
    let CACHE: Vec<f64> = initialize_cache();

    fn initialize_cache() -> Vec<f64> {
        // Called once at program startup
    }
}
```

### 11.2 Main Entry Point

```groklang
fn main() -> () {
    // User entry point
    println!("Hello, World!");
}

// Implicit wrapper:
// 1. Initialize all modules
// 2. Call main()
// 3. Drop all values in reverse order
// 4. Exit
```

---

## 12. Performance Considerations

### 12.1 Zero-Cost Abstractions

Memory operations compile to efficient code:

```groklang
fn sum(items: &[i32]) -> i32 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    total
}
// Compiles to: Load, Add, Store (in a loop)
// No allocation, no boxing, no indirection
```

### 12.2 Inlining

Small functions automatically inlined:

```groklang
fn add(a: i32, b: i32) -> i32 {
    a + b
}
// Inlined as: add_inst_1: LOAD a; LOAD b; ADD; STORE result
```

### 12.3 Stack Allocation Preferred

Heap allocation minimized:

- Small types on stack
- Large types in stack (if `Sized`)
- Heap only when needed (Vec, String, etc.)

---

## 13. Validation Criteria

Implementation must satisfy:

- [ ] Stack allocation works for all primitives
- [ ] Move semantics correctly transfers ownership
- [ ] Borrow checker prevents use-after-free
- [ ] Borrow checker prevents data races
- [ ] Drop trait called on scope exit
- [ ] Reference counting works for shared ownership
- [ ] Threads spawn and join correctly
- [ ] Message passing works cross-thread
- [ ] Mutexes provide mutual exclusion
- [ ] Lifetimes elision rules applied correctly
- [ ] Generic monomorphization correct
- [ ] AI-GC doesn't cause memory leaks

---

## 14. Related Documents

- [01-Architecture-Overview.md](01-Architecture-Overview.md) - System architecture
- [02-Type-System-Specification.md](02-Type-System-Specification.md) - Type system
- [05-AI-Integration-Specification.md](05-AI-Integration-Specification.md) - AI deadlock detection
