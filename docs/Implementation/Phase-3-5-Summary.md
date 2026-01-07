# Phase 3, 4, 5: Code Generation, Runtime, and FFI Summary

**Document Version**: 1.0  
**Date**: January 7, 2026  
**Scope**: Code generation, runtime, FFI implementation overview

---

## Phase 3: Code Generation (3-4 weeks)

### 3.1 IR (Intermediate Representation)

```python
# Simple bytecode-like IR
class IRInstruction:
    def __init__(self, opcode: str, args: list):
        self.opcode = opcode
        self.args = args

class IRBlock:
    def __init__(self, label: str):
        self.label = label
        self.instructions = []

class IRFunction:
    def __init__(self, name: str, params: list, blocks: list):
        self.name = name
        self.params = params
        self.blocks = blocks
```

### 3.2 Codegen from Typed AST

```python
class CodeGenerator:
    def generate(self, typed_ast) -> list:
        """Generate IR from typed AST"""
        functions = []
        for item in typed_ast:
            if isinstance(item, FunctionDef):
                ir_func = self.gen_function(item)
                functions.append(ir_func)
        return functions

    def gen_function(self, func: FunctionDef):
        ir_blocks = [IRBlock("entry")]
        self.gen_expr(func.body, ir_blocks[0])
        return IRFunction(func.name, func.params, ir_blocks)

    def gen_expr(self, expr, block: IRBlock):
        if isinstance(expr, IntLiteral):
            block.instructions.append(IRInstruction("PUSH_INT", [expr.value]))
        elif isinstance(expr, BinaryOp):
            self.gen_expr(expr.left, block)
            self.gen_expr(expr.right, block)
            block.instructions.append(IRInstruction("ADD" if expr.op == '+' else "SUB", []))
        # ... more cases
```

### 3.3 Backend Targets

- **Bytecode VM**: Direct execution (fast development cycle)
- **LLVM**: Native code (high performance)
- **JavaScript**: Via transpilation (future)

### 3.4 Deliverables

- [x] IR design and representation
- [x] AST-to-IR lowering
- [x] Bytecode backend
- [x] LLVM integration (via llvmlite)
- [x] Optimization passes
- [ ] Validation checklist

---

## Phase 4: Runtime and Concurrency (4-5 weeks)

### 4.1 Memory Management

```python
class MemoryManager:
    def __init__(self):
        self.allocations = {}  # address -> (value, refcount)
        self.next_addr = 1000

    def allocate(self, value):
        """Allocate on heap"""
        addr = self.next_addr
        self.allocations[addr] = [value, 1]  # [value, refcount]
        self.next_addr += 1
        return addr

    def clone_ref(self, addr):
        """Increment refcount"""
        self.allocations[addr][1] += 1

    def drop_ref(self, addr):
        """Decrement refcount, deallocate if zero"""
        self.allocations[addr][1] -= 1
        if self.allocations[addr][1] == 0:
            del self.allocations[addr]
```

### 4.2 Borrow Checker Runtime

```python
class BorrowTracker:
    def __init__(self):
        self.borrows = {}  # addr -> set of borrow refs

    def immutable_borrow(self, addr):
        """Create immutable borrow"""
        if addr not in self.borrows:
            self.borrows[addr] = set()
        borrow_id = id(object())  # Unique ID
        self.borrows[addr].add(('immutable', borrow_id))
        return borrow_id

    def mutable_borrow(self, addr):
        """Create mutable borrow (exclusive)"""
        if addr in self.borrows and len(self.borrows[addr]) > 0:
            raise RuntimeError("Cannot create mutable borrow while other borrows exist")
        borrow_id = id(object())
        self.borrows[addr].add(('mutable', borrow_id))
        return borrow_id

    def release_borrow(self, addr, borrow_id):
        """End borrow"""
        if addr in self.borrows:
            self.borrows[addr] = {b for b in self.borrows[addr] if b[1] != borrow_id}
```

### 4.3 Thread Runtime

```python
import threading
from queue import Queue

class ThreadRuntime:
    def __init__(self):
        self.threads = {}
        self.thread_id = 0

    def spawn(self, func, args):
        """Spawn lightweight thread"""
        tid = self.thread_id
        self.thread_id += 1

        def wrapper():
            try:
                result = func(*args)
                return result
            except Exception as e:
                return f"Error: {e}"

        thread = threading.Thread(target=wrapper)
        self.threads[tid] = thread
        thread.start()
        return tid

    def join(self, tid):
        """Wait for thread completion"""
        self.threads[tid].join()

class ChannelRuntime:
    def __init__(self):
        self.channels = {}

    def create_channel(self):
        """Create message channel"""
        cid = id(object())
        self.channels[cid] = Queue()
        return cid

    def send(self, cid, message):
        """Send message"""
        self.channels[cid].put(message)

    def recv(self, cid):
        """Receive message (blocking)"""
        return self.channels[cid].get()
```

### 4.4 Actor Runtime (Optional)

```python
class Actor:
    def __init__(self, name: str):
        self.name = name
        self.mailbox = Queue()
        self.state = {}

    def send_message(self, message):
        """Send message to actor"""
        self.mailbox.put(message)

    def receive(self):
        """Receive message (blocking)"""
        return self.mailbox.get()

    def run(self):
        """Actor event loop"""
        while True:
            msg = self.receive()
            if msg == 'EXIT':
                break
            # Process message
            self.handle_message(msg)

    def handle_message(self, msg):
        """Override in subclass"""
        pass

class ActorRuntime:
    def __init__(self):
        self.actors = {}

    def create_actor(self, actor_class, name):
        actor = actor_class(name)
        self.actors[name] = actor

        thread = threading.Thread(target=actor.run)
        thread.start()
        return actor
```

### 4.5 Deadlock Detector

```python
class DeadlockDetector:
    def __init__(self, timeout: float = 5.0):
        self.timeout = timeout
        self.lock_graph = {}  # thread -> locks held

    def detect_cycle(self):
        """Check for circular wait"""
        for thread_id, locks in self.lock_graph.items():
            if self.has_cycle(thread_id, locks, set()):
                return True
        return False

    def has_cycle(self, thread_id, locks, visited):
        """DFS to detect cycle in lock graph"""
        if thread_id in visited:
            return True

        visited.add(thread_id)
        for lock in locks:
            # Find threads waiting on this lock
            for other_id, other_locks in self.lock_graph.items():
                if other_id != thread_id:
                    if lock in other_locks:
                        if self.has_cycle(other_id, other_locks, visited.copy()):
                            return True
        return False
```

### 4.6 Deliverables

- [x] Memory allocator
- [x] Reference counting
- [x] Borrow checker enforcement
- [x] Thread runtime (spawn, join)
- [x] Channel/message-passing
- [x] Actor framework (basic)
- [x] Deadlock detector
- [ ] Validation checklist

---

## Phase 5: FFI and AI Integration (2-3 weeks)

### 5.1 FFI Type Marshaling

```python
class TypeMarshaler:
    def grok_to_c(self, value, type_: Type):
        """Convert GrokLang value to C-compatible"""
        if isinstance(type_, PrimitiveType):
            if type_.name == "i32":
                return int(value)
            elif type_.name == "f64":
                return float(value)
            elif type_.name == "str":
                return value.encode('utf-8') + b'\x00'
        elif isinstance(type_, GenericType):
            if type_.name == "Vec":
                return (len(value), value)  # (ptr, len)
        return value

    def c_to_grok(self, value, type_: Type):
        """Convert C value back to GrokLang"""
        if isinstance(type_, PrimitiveType):
            if type_.name == "i32":
                return int(value)
            elif type_.name == "f64":
                return float(value)
        return value
```

### 5.2 Python FFI

```python
import ctypes
import importlib

class PythonFFI:
    def __init__(self):
        self.modules = {}

    def call_python(self, module_name: str, func_name: str, args):
        """Call Python function from Grok"""
        if module_name not in self.modules:
            self.modules[module_name] = importlib.import_module(module_name)

        module = self.modules[module_name]
        func = getattr(module, func_name)

        # Marshal arguments and call
        result = func(*args)
        return result

    def export_function(self, func_name: str, func):
        """Export Grok function to Python"""
        # Register in a global registry
        GROK_EXPORTS[func_name] = func

GROK_EXPORTS = {}
```

### 5.3 C FFI via ctypes

```python
class CFFIBinding:
    def __init__(self, lib_path: str):
        self.lib = ctypes.CDLL(lib_path)

    def call(self, func_name: str, arg_types: list, return_type, args):
        """Call C function"""
        func = getattr(self.lib, func_name)
        func.argtypes = arg_types
        func.restype = return_type
        return func(*args)

# Usage in generated code
c_lib = CFFIBinding("libc.so.6")
result = c_lib.call("strlen", [ctypes.c_char_p], ctypes.c_int, [b"hello"])
```

### 5.4 AI Decorator Execution

```python
class DecoratorExecutor:
    def __init__(self, ai_service):
        self.ai_service = ai_service

    def execute_ai_optimize(self, func, level="intermediate"):
        """Execute ai_optimize at runtime"""
        # Measure baseline
        baseline_time = self.benchmark(func)

        # Request AI optimization
        response = self.ai_service.call({
            'operation': 'optimize',
            'input': func_to_code(func),
            'parameters': {'level': level}
        })

        if response['success']:
            optimized_code = response['output']
            optimized_func = compile_function(optimized_code)

            # Benchmark optimized
            opt_time = self.benchmark(optimized_func)

            # Use if faster
            if opt_time < baseline_time:
                return optimized_func

        return func  # Fall back to original

    def benchmark(self, func, iterations=100):
        """Benchmark function execution time"""
        import time
        start = time.time()
        for _ in range(iterations):
            func()
        return time.time() - start
```

### 5.5 Deliverables

- [x] Type marshaling layer
- [x] Python FFI bridge
- [x] C FFI via ctypes
- [x] Bidirectional calling
- [x] Exception marshaling
- [x] AI decorator execution
- [ ] Validation checklist

---

## Implementation Timeline

```
Phase 1 (Lexer/Parser):   ████████░░ 2-3 weeks
Phase 2 (Type/Decorator): ██████████ 4-5 weeks
Phase 3 (Codegen):        ████████░░ 3-4 weeks
Phase 4 (Runtime):        ██████████ 4-5 weeks
Phase 5 (FFI/AI):         ████████░░ 2-3 weeks

Total:                     ~18-22 weeks
```

---

## Key Implementation Considerations

### Performance

- Use caching where possible (type checking, codegen)
- Lazy evaluation of decorators
- JIT compilation for hot paths

### Correctness

- Extensive test coverage (>90%)
- Formal verification for type checker
- Symbolic execution for optimization validation

### Debugging

- Detailed error messages with suggestions
- Stack traces with source locations
- Debugger hooks for IDE integration

### Extensibility

- Plugin system for custom decorators
- Configurable AI backends
- User-defined optimization passes

---

## Validation Approach

Each phase includes comprehensive validation:

- Unit tests for each module
- Integration tests between phases
- Regression tests for correctness
- Performance benchmarks
- Real-world code samples

See individual phase validation documents for specific checklists.
