// grok/src/optimizations.rs
//
// VM Optimizations Module
// Implements: Hot-path JIT, Bytecode Specialization, Inline Caching, Tail Call Optimization

use crate::ir::{IRFunction, IRBlock, Opcode};
#[cfg(test)]
use crate::ir::IRInstruction;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// 1. BYTECODE SPECIALIZATION
// ============================================================================
// Specialized opcodes for common patterns to reduce runtime type checking

#[derive(Debug, Clone)]
pub enum SpecializedOpcode {
    // Original opcodes (unspecialized)
    Generic(Opcode),
    
    // Specialized integer operations (no type checking needed at runtime)
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntLt,
    IntGt,
    IntLe,
    IntGe,
    IntEq,
    IntNe,
    
    // Specialized loads (with cached slot indices)
    LoadLocalFast(usize),      // Direct slot access instead of HashMap lookup
    StoreLocalFast(usize),     // Direct slot access instead of HashMap lookup
    
    // Tail call optimization
    TailCall(String, usize),   // Reuse current frame instead of allocating new one
    
    // Inlined constants
    PushSmallInt(i64),         // For small ints that fit in the opcode
}

#[derive(Debug, Clone)]
pub struct SpecializedInstruction {
    pub opcode: SpecializedOpcode,
}

#[derive(Debug, Clone)]
pub struct SpecializedBlock {
    pub label: String,
    pub instructions: Vec<SpecializedInstruction>,
}

#[derive(Debug, Clone)]
pub struct SpecializedFunction {
    pub name: String,
    pub params: Vec<String>,
    pub param_slots: HashMap<String, usize>,  // Fast slot lookup
    pub blocks: Vec<SpecializedBlock>,
    pub is_hot: bool,
    pub call_count: u64,
}

// ============================================================================
// 2. INLINE CACHING
// ============================================================================
// Cache function lookups and field accesses to avoid HashMap lookups

#[derive(Debug, Clone)]
pub struct InlineCache {
    // Function call cache: maps call site ID to resolved function pointer
    pub function_cache: HashMap<usize, Arc<SpecializedFunction>>,
    
    // Field access cache: maps (type_name, field_name) to field offset
    pub field_cache: HashMap<(String, String), usize>,
    
    // Variable slot cache: maps variable name to local slot index
    pub slot_cache: HashMap<String, usize>,
}

impl InlineCache {
    pub fn new() -> Self {
        Self {
            function_cache: HashMap::new(),
            field_cache: HashMap::new(),
            slot_cache: HashMap::new(),
        }
    }
    
    pub fn cache_function(&mut self, call_site: usize, func: Arc<SpecializedFunction>) {
        self.function_cache.insert(call_site, func);
    }
    
    pub fn get_cached_function(&self, call_site: usize) -> Option<&Arc<SpecializedFunction>> {
        self.function_cache.get(&call_site)
    }
    
    pub fn cache_field(&mut self, type_name: String, field_name: String, offset: usize) {
        self.field_cache.insert((type_name, field_name), offset);
    }
    
    pub fn get_cached_field(&self, type_name: &str, field_name: &str) -> Option<usize> {
        self.field_cache.get(&(type_name.to_string(), field_name.to_string())).copied()
    }
    
    pub fn cache_slot(&mut self, var_name: String, slot: usize) {
        self.slot_cache.insert(var_name, slot);
    }
    
    pub fn get_cached_slot(&self, var_name: &str) -> Option<usize> {
        self.slot_cache.get(var_name).copied()
    }
}

// ============================================================================
// 3. HOT PATH DETECTION
// ============================================================================
// Track function call counts and identify hot functions for JIT compilation

pub const HOT_THRESHOLD: u64 = 100;  // Number of calls before considered "hot"

#[derive(Debug)]
pub struct HotPathTracker {
    pub call_counts: HashMap<String, u64>,
    pub hot_functions: Vec<String>,
}

impl HotPathTracker {
    pub fn new() -> Self {
        Self {
            call_counts: HashMap::new(),
            hot_functions: Vec::new(),
        }
    }
    
    pub fn record_call(&mut self, func_name: &str) -> bool {
        let count = self.call_counts.entry(func_name.to_string()).or_insert(0);
        *count += 1;
        
        if *count == HOT_THRESHOLD && !self.hot_functions.contains(&func_name.to_string()) {
            self.hot_functions.push(func_name.to_string());
            true  // Just became hot
        } else {
            false
        }
    }
    
    pub fn is_hot(&self, func_name: &str) -> bool {
        self.hot_functions.contains(&func_name.to_string())
    }
    
    pub fn get_call_count(&self, func_name: &str) -> u64 {
        self.call_counts.get(func_name).copied().unwrap_or(0)
    }
}

// ============================================================================
// 4. TAIL CALL OPTIMIZATION
// ============================================================================
// Detect and optimize tail calls to prevent stack overflow

pub fn is_tail_call(block: &IRBlock, instr_idx: usize) -> bool {
    // A call is a tail call if:
    // 1. It's followed immediately by a Ret
    // 2. Or it's the last instruction before Ret in the block
    
    let instructions = &block.instructions;
    if instr_idx >= instructions.len() {
        return false;
    }
    
    if let Opcode::Call(_, _) = &instructions[instr_idx].opcode {
        // Check if next instruction is Ret
        if instr_idx + 1 < instructions.len() {
            if let Opcode::Ret = &instructions[instr_idx + 1].opcode {
                return true;
            }
        }
    }
    
    false
}

// ============================================================================
// 5. BYTECODE OPTIMIZER
// ============================================================================
// Transforms generic IR into specialized bytecode

pub struct BytecodeOptimizer {
    pub inline_cache: InlineCache,
    call_site_counter: usize,
}

impl BytecodeOptimizer {
    pub fn new() -> Self {
        Self {
            inline_cache: InlineCache::new(),
            call_site_counter: 0,
        }
    }
    
    pub fn optimize(&mut self, func: &IRFunction) -> SpecializedFunction {
        let mut param_slots = HashMap::new();
        for (idx, param) in func.params.iter().enumerate() {
            param_slots.insert(param.clone(), idx);
            self.inline_cache.cache_slot(param.clone(), idx);
        }
        
        let mut specialized_blocks = Vec::new();
        
        for block in &func.blocks {
            let specialized_block = self.optimize_block(block, &param_slots);
            specialized_blocks.push(specialized_block);
        }
        
        SpecializedFunction {
            name: func.name.clone(),
            params: func.params.clone(),
            param_slots,
            blocks: specialized_blocks,
            is_hot: false,
            call_count: 0,
        }
    }
    
    fn optimize_block(&mut self, block: &IRBlock, param_slots: &HashMap<String, usize>) -> SpecializedBlock {
        let mut instructions = Vec::new();
        
        for (idx, instr) in block.instructions.iter().enumerate() {
            let specialized = self.specialize_instruction(&instr.opcode, block, idx, param_slots);
            instructions.push(SpecializedInstruction { opcode: specialized });
        }
        
        SpecializedBlock {
            label: block.label.clone(),
            instructions,
        }
    }
    
    fn specialize_instruction(
        &mut self,
        opcode: &Opcode,
        block: &IRBlock,
        idx: usize,
        param_slots: &HashMap<String, usize>,
    ) -> SpecializedOpcode {
        match opcode {
            // Specialize arithmetic operations
            Opcode::Add => SpecializedOpcode::IntAdd,
            Opcode::Sub => SpecializedOpcode::IntSub,
            Opcode::Mul => SpecializedOpcode::IntMul,
            Opcode::Div => SpecializedOpcode::IntDiv,
            
            // Specialize comparisons
            Opcode::Lt => SpecializedOpcode::IntLt,
            Opcode::Gt => SpecializedOpcode::IntGt,
            Opcode::Le => SpecializedOpcode::IntLe,
            Opcode::Ge => SpecializedOpcode::IntGe,
            Opcode::Eq => SpecializedOpcode::IntEq,
            Opcode::Ne => SpecializedOpcode::IntNe,
            
            // Specialize variable access with slot caching
            Opcode::LoadVar(name) => {
                if let Some(&slot) = param_slots.get(name) {
                    SpecializedOpcode::LoadLocalFast(slot)
                } else if let Some(slot) = self.inline_cache.get_cached_slot(name) {
                    SpecializedOpcode::LoadLocalFast(slot)
                } else {
                    SpecializedOpcode::Generic(opcode.clone())
                }
            }
            
            Opcode::StoreVar(name) => {
                let next_slot = self.inline_cache.slot_cache.len();
                let slot = *self.inline_cache.slot_cache.entry(name.clone()).or_insert(next_slot);
                SpecializedOpcode::StoreLocalFast(slot)
            }
            
            // Optimize small integer constants
            Opcode::PushInt(v) if *v >= -128 && *v <= 127 => {
                SpecializedOpcode::PushSmallInt(*v)
            }
            
            // Tail call optimization
            Opcode::Call(name, arg_count) => {
                if is_tail_call(block, idx) {
                    SpecializedOpcode::TailCall(name.clone(), *arg_count)
                } else {
                    self.call_site_counter += 1;
                    SpecializedOpcode::Generic(opcode.clone())
                }
            }
            
            // Keep other opcodes generic
            _ => SpecializedOpcode::Generic(opcode.clone()),
        }
    }
}

// ============================================================================
// 6. OPTIMIZED VM EXECUTION
// ============================================================================
// Fast execution path using specialized bytecode

use crate::vm::Value;

/// Fast local variable storage using slots instead of HashMap
pub struct FastLocals {
    slots: Vec<Value>,
}

impl FastLocals {
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: vec![Value::Unit; capacity.max(16)],
        }
    }
    
    #[inline(always)]
    pub fn get(&self, slot: usize) -> &Value {
        &self.slots[slot]
    }
    
    #[inline(always)]
    pub fn set(&mut self, slot: usize, value: Value) {
        if slot >= self.slots.len() {
            self.slots.resize(slot + 1, Value::Unit);
        }
        self.slots[slot] = value;
    }
}

/// Optimized execution context
pub struct OptimizedContext {
    pub stack: Vec<Value>,
    pub locals: FastLocals,
    pub hot_tracker: HotPathTracker,
    pub inline_cache: InlineCache,
}

impl OptimizedContext {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            locals: FastLocals::new(32),
            hot_tracker: HotPathTracker::new(),
            inline_cache: InlineCache::new(),
        }
    }
    
    #[inline(always)]
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    
    #[inline(always)]
    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
    
    /// Execute a specialized instruction
    #[inline]
    pub fn execute_specialized(&mut self, opcode: &SpecializedOpcode) -> Result<(), String> {
        match opcode {
            SpecializedOpcode::IntAdd => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Int(a + b));
                    Ok(())
                } else {
                    Err("Type error in IntAdd".to_string())
                }
            }
            
            SpecializedOpcode::IntSub => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Int(a - b));
                    Ok(())
                } else {
                    Err("Type error in IntSub".to_string())
                }
            }
            
            SpecializedOpcode::IntMul => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Int(a * b));
                    Ok(())
                } else {
                    Err("Type error in IntMul".to_string())
                }
            }
            
            SpecializedOpcode::IntDiv => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    if b == 0 {
                        return Err("Division by zero".to_string());
                    }
                    self.push(Value::Int(a / b));
                    Ok(())
                } else {
                    Err("Type error in IntDiv".to_string())
                }
            }
            
            SpecializedOpcode::IntLt => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Bool(a < b));
                    Ok(())
                } else {
                    Err("Type error in IntLt".to_string())
                }
            }
            
            SpecializedOpcode::IntGt => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Bool(a > b));
                    Ok(())
                } else {
                    Err("Type error in IntGt".to_string())
                }
            }
            
            SpecializedOpcode::IntLe => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Bool(a <= b));
                    Ok(())
                } else {
                    Err("Type error in IntLe".to_string())
                }
            }
            
            SpecializedOpcode::IntGe => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Bool(a >= b));
                    Ok(())
                } else {
                    Err("Type error in IntGe".to_string())
                }
            }
            
            SpecializedOpcode::IntEq => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Bool(a == b));
                    Ok(())
                } else {
                    Err("Type error in IntEq".to_string())
                }
            }
            
            SpecializedOpcode::IntNe => {
                let b = self.pop().ok_or("Stack underflow")?;
                let a = self.pop().ok_or("Stack underflow")?;
                if let (Value::Int(a), Value::Int(b)) = (a, b) {
                    self.push(Value::Bool(a != b));
                    Ok(())
                } else {
                    Err("Type error in IntNe".to_string())
                }
            }
            
            SpecializedOpcode::LoadLocalFast(slot) => {
                let value = self.locals.get(*slot).clone();
                self.push(value);
                Ok(())
            }
            
            SpecializedOpcode::StoreLocalFast(slot) => {
                let value = self.pop().ok_or("Stack underflow")?;
                self.locals.set(*slot, value);
                Ok(())
            }
            
            SpecializedOpcode::PushSmallInt(v) => {
                self.push(Value::Int(*v));
                Ok(())
            }
            
            SpecializedOpcode::TailCall(_, _) => {
                // Handled specially in the VM loop
                Ok(())
            }
            
            SpecializedOpcode::Generic(_) => {
                // Fallback to original VM execution
                Ok(())
            }
        }
    }
}

// ============================================================================
// 7. OPTIMIZED VM
// ============================================================================

pub struct OptimizedVM {
    pub context: OptimizedContext,
    pub functions: HashMap<String, SpecializedFunction>,
    pub optimizer: BytecodeOptimizer,
}

impl OptimizedVM {
    pub fn new() -> Self {
        Self {
            context: OptimizedContext::new(),
            functions: HashMap::new(),
            optimizer: BytecodeOptimizer::new(),
        }
    }
    
    pub fn load_program(&mut self, ir_functions: &[IRFunction]) {
        for func in ir_functions {
            let specialized = self.optimizer.optimize(func);
            self.functions.insert(func.name.clone(), specialized);
        }
    }
    
    pub fn execute(&mut self, func_name: &str, args: Vec<Value>) -> Result<Value, String> {
        // Initialize locals with arguments
        for (idx, arg) in args.into_iter().enumerate() {
            self.context.locals.set(idx, arg);
        }
        
        self.execute_function(func_name)
    }
    
    fn execute_function(&mut self, func_name: &str) -> Result<Value, String> {
        // Track hot paths
        let became_hot = self.context.hot_tracker.record_call(func_name);
        if became_hot {
            // In a real implementation, we would trigger JIT compilation here
            // For now, just mark the function as hot
            if let Some(func) = self.functions.get_mut(func_name) {
                func.is_hot = true;
            }
        }
        
        let func = self.functions.get(func_name)
            .ok_or_else(|| format!("Function {} not found", func_name))?
            .clone();
        
        let mut block_idx = 0;
        let mut instr_idx = 0;
        
        loop {
            if block_idx >= func.blocks.len() {
                break;
            }
            
            let block = &func.blocks[block_idx];
            
            if instr_idx >= block.instructions.len() {
                block_idx += 1;
                instr_idx = 0;
                continue;
            }
            
            let instr = &block.instructions[instr_idx];
            instr_idx += 1;
            
            match &instr.opcode {
                SpecializedOpcode::TailCall(callee_name, arg_count) => {
                    // Tail call optimization: reuse current frame
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.context.pop().ok_or("Stack underflow in TailCall")?);
                    }
                    args.reverse();
                    
                    // Reset locals and reinitialize with new args
                    self.context.locals = FastLocals::new(args.len());
                    for (idx, arg) in args.into_iter().enumerate() {
                        self.context.locals.set(idx, arg);
                    }
                    
                    // Jump to the callee function (tail call - no new frame)
                    if callee_name == func_name {
                        // Self-recursive tail call - just restart the function
                        block_idx = 0;
                        instr_idx = 0;
                        continue;
                    } else {
                        // Tail call to different function
                        return self.execute_function(callee_name);
                    }
                }
                
                SpecializedOpcode::Generic(opcode) => {
                    // Handle generic opcodes
                    match opcode {
                        crate::ir::Opcode::PushInt(v) => {
                            self.context.push(Value::Int(*v));
                        }
                        crate::ir::Opcode::PushBool(v) => {
                            self.context.push(Value::Bool(*v));
                        }
                        crate::ir::Opcode::LoadVar(name) => {
                            // Try fast path first
                            if let Some(&slot) = func.param_slots.get(name) {
                                let value = self.context.locals.get(slot).clone();
                                self.context.push(value);
                            } else if let Some(slot) = self.context.inline_cache.get_cached_slot(name) {
                                let value = self.context.locals.get(slot).clone();
                                self.context.push(value);
                            } else {
                                return Err(format!("Variable {} not found", name));
                            }
                        }
                        crate::ir::Opcode::Call(callee_name, arg_count) => {
                            let mut args = Vec::new();
                            for _ in 0..*arg_count {
                                args.push(self.context.pop().ok_or("Stack underflow in Call")?);
                            }
                            args.reverse();
                            
                            // Save current locals
                            let saved_locals = std::mem::replace(
                                &mut self.context.locals,
                                FastLocals::new(args.len())
                            );
                            
                            // Set up new locals with args
                            for (idx, arg) in args.into_iter().enumerate() {
                                self.context.locals.set(idx, arg);
                            }
                            
                            // Execute callee
                            let result = self.execute_function(callee_name)?;
                            
                            // Restore locals
                            self.context.locals = saved_locals;
                            
                            // Push result
                            self.context.push(result);
                        }
                        crate::ir::Opcode::Ret => {
                            // Return the top of stack
                            let result = self.context.pop().unwrap_or(Value::Unit);
                            return Ok(result);
                        }
                        crate::ir::Opcode::Jmp(label) => {
                            // Find block with matching label
                            block_idx = func.blocks.iter()
                                .position(|b| b.label == *label)
                                .ok_or_else(|| format!("Label {} not found", label))?;
                            instr_idx = 0;
                            continue;
                        }
                        crate::ir::Opcode::JmpIfFalse(label) => {
                            let cond = self.context.pop().ok_or("Stack underflow in JmpIfFalse")?;
                            if cond == Value::Bool(false) {
                                block_idx = func.blocks.iter()
                                    .position(|b| b.label == *label)
                                    .ok_or_else(|| format!("Label {} not found", label))?;
                                instr_idx = 0;
                                continue;
                            }
                        }
                        _ => {
                            // Other generic opcodes
                        }
                    }
                }
                
                // Execute specialized instructions
                other => {
                    self.context.execute_specialized(other)?;
                }
            }
        }
        
        Ok(self.context.pop().unwrap_or(Value::Unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bytecode_specialization() {
        let mut optimizer = BytecodeOptimizer::new();
        
        let ir_func = IRFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            blocks: vec![IRBlock {
                label: "entry".to_string(),
                instructions: vec![
                    IRInstruction { opcode: Opcode::LoadVar("a".to_string()) },
                    IRInstruction { opcode: Opcode::LoadVar("b".to_string()) },
                    IRInstruction { opcode: Opcode::Add },
                    IRInstruction { opcode: Opcode::Ret },
                ],
            }],
        };
        
        let specialized = optimizer.optimize(&ir_func);
        
        // Check that opcodes are specialized
        assert!(matches!(specialized.blocks[0].instructions[0].opcode, SpecializedOpcode::LoadLocalFast(0)));
        assert!(matches!(specialized.blocks[0].instructions[1].opcode, SpecializedOpcode::LoadLocalFast(1)));
        assert!(matches!(specialized.blocks[0].instructions[2].opcode, SpecializedOpcode::IntAdd));
    }
    
    #[test]
    fn test_tail_call_detection() {
        let block = IRBlock {
            label: "test".to_string(),
            instructions: vec![
                IRInstruction { opcode: Opcode::LoadVar("n".to_string()) },
                IRInstruction { opcode: Opcode::Call("fib".to_string(), 1) },
                IRInstruction { opcode: Opcode::Ret },
            ],
        };
        
        assert!(!is_tail_call(&block, 0)); // LoadVar is not a call
        assert!(is_tail_call(&block, 1));   // Call followed by Ret
        assert!(!is_tail_call(&block, 2));  // Ret is not a call
    }
    
    #[test]
    fn test_hot_path_tracking() {
        let mut tracker = HotPathTracker::new();
        
        for _ in 0..HOT_THRESHOLD - 1 {
            assert!(!tracker.record_call("test_func"));
        }
        
        // This should trigger hot detection
        assert!(tracker.record_call("test_func"));
        assert!(tracker.is_hot("test_func"));
        
        // Subsequent calls should not re-trigger
        assert!(!tracker.record_call("test_func"));
    }
    
    #[test]
    fn test_inline_cache() {
        let mut cache = InlineCache::new();
        
        cache.cache_slot("x".to_string(), 0);
        cache.cache_slot("y".to_string(), 1);
        
        assert_eq!(cache.get_cached_slot("x"), Some(0));
        assert_eq!(cache.get_cached_slot("y"), Some(1));
        assert_eq!(cache.get_cached_slot("z"), None);
        
        cache.cache_field("Point".to_string(), "x".to_string(), 0);
        assert_eq!(cache.get_cached_field("Point", "x"), Some(0));
    }
    
    #[test]
    fn test_optimized_vm_simple() {
        let mut vm = OptimizedVM::new();
        
        // Create a simple add function: fn add(a, b) { return a + b }
        let ir_funcs = vec![IRFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            blocks: vec![IRBlock {
                label: "entry".to_string(),
                instructions: vec![
                    IRInstruction { opcode: Opcode::LoadVar("a".to_string()) },
                    IRInstruction { opcode: Opcode::LoadVar("b".to_string()) },
                    IRInstruction { opcode: Opcode::Add },
                    IRInstruction { opcode: Opcode::Ret },
                ],
            }],
        }];
        
        vm.load_program(&ir_funcs);
        
        let result = vm.execute("add", vec![Value::Int(10), Value::Int(32)]);
        assert_eq!(result, Ok(Value::Int(42)));
    }
    
    #[test]
    fn test_fast_locals() {
        let mut locals = FastLocals::new(4);
        
        locals.set(0, Value::Int(42));
        locals.set(1, Value::Bool(true));
        
        assert_eq!(locals.get(0), &Value::Int(42));
        assert_eq!(locals.get(1), &Value::Bool(true));
        
        // Test dynamic growth
        locals.set(10, Value::Int(100));
        assert_eq!(locals.get(10), &Value::Int(100));
    }
}
