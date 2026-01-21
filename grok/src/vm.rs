// grok/src/vm.rs

use crate::ir::{IRFunction, Opcode};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Object(usize), // Index into the heap
    Unit,
}

#[derive(Debug, Clone)]
pub enum HeapObject {
    Str(String),
    Struct(String, HashMap<String, Value>),
}

pub struct Heap {
    objects: Vec<Option<HeapObject>>,
    marked: Vec<bool>,
    free_list: Vec<usize>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            marked: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn alloc(&mut self, obj: HeapObject) -> usize {
        if let Some(idx) = self.free_list.pop() {
            self.objects[idx] = Some(obj);
            self.marked[idx] = false;
            idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(obj));
            self.marked.push(false);
            idx
        }
    }

    pub fn get(&self, idx: usize) -> Option<&HeapObject> {
        self.objects.get(idx).and_then(|o| o.as_ref())
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut HeapObject> {
        self.objects.get_mut(idx).and_then(|o| o.as_mut())
    }

    pub fn gc(&mut self, roots: &[Value], frames: &[Frame], globals: &HashMap<String, Value>) {
        // 1. Mark
        self.marked.fill(false);
        for root in roots {
            self.mark_value(root);
        }
        for frame in frames {
            for val in frame.locals.values() {
                self.mark_value(val);
            }
        }
        for val in globals.values() {
            self.mark_value(val);
        }

        // 2. Sweep
        for i in 0..self.objects.len() {
            if self.objects[i].is_some() && !self.marked[i] {
                self.objects[i] = None;
                self.free_list.push(i);
            }
        }
    }

    fn mark_value(&mut self, val: &Value) {
        if let Value::Object(idx) = val {
            if !self.marked[*idx] {
                self.marked[*idx] = true;
                // Recursively mark fields of structs
                if let Some(Some(HeapObject::Struct(_, fields))) = self.objects.get(*idx) {
                    let fields = fields.clone(); // Avoid borrow issues while recursing
                    for f_val in fields.values() {
                        self.mark_value(f_val);
                    }
                }
            }
        }
    }
}

impl Value {
    pub fn into_int(self) -> Option<i64> {
        if let Value::Int(v) = self { Some(v) } else { None }
    }
}

pub struct Frame {
    pub func_name: String,
    pub current_block_idx: usize,
    pub current_instr_idx: usize,
    pub locals: HashMap<String, Value>,
}

pub struct VM {
    stack: Vec<Value>,
    call_stack: Vec<Frame>,
    functions: HashMap<String, IRFunction>,
    globals: HashMap<String, Value>,
    pub heap: Heap,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            functions: HashMap::new(),
            globals: HashMap::new(),
            heap: Heap::new(),
        }
    }

    pub fn load_program(&mut self, functions: &[IRFunction]) {
        for func in functions {
            self.functions.insert(func.name.clone(), func.clone());
        }
    }

    pub fn execute(&mut self, func_name: &str) -> Result<Value, String> {
        self.call_stack.push(Frame {
            func_name: func_name.to_string(),
            current_block_idx: 0,
            current_instr_idx: 0,
            locals: HashMap::new(),
        });

        // Simple GC counter
        let mut gc_timer = 0;

        while !self.call_stack.is_empty() {
            gc_timer += 1;
            if gc_timer > 1000 {
                self.heap.gc(&self.stack, &self.call_stack, &self.globals);
                gc_timer = 0;
            }

            let (instr, func) = {
                let frame = self.call_stack.last().unwrap();
                let func = self.functions.get(&frame.func_name).unwrap();
                let block = &func.blocks[frame.current_block_idx];
                if frame.current_instr_idx >= block.instructions.len() {
                    if frame.current_block_idx + 1 < func.blocks.len() {
                        let frame = self.call_stack.last_mut().unwrap();
                        frame.current_block_idx += 1;
                        frame.current_instr_idx = 0;
                        continue;
                    } else {
                        return Err("Unexpected end of function".to_string());
                    }
                }
                (&block.instructions[frame.current_instr_idx], func.clone())
            };

            let frame = self.call_stack.last_mut().unwrap();
            frame.current_instr_idx += 1;

            match &instr.opcode {
                Opcode::PushInt(v) => self.stack.push(Value::Int(*v)),
                Opcode::PushBool(v) => self.stack.push(Value::Bool(*v)),
                Opcode::LoadVar(name) => {
                    let val = frame.locals.get(name).cloned().unwrap_or(Value::Unit);
                    self.stack.push(val);
                }
                Opcode::StoreVar(name) => {
                    let val = self.stack.pop().unwrap_or(Value::Unit);
                    frame.locals.insert(name.clone(), val);
                }
                Opcode::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Some(Value::Int(va)), Some(Value::Int(vb))) => self.stack.push(Value::Int(va + vb)),
                        _ => return Err("Invalid types for Add".to_string()),
                    }
                }
                Opcode::Sub => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Some(Value::Int(va)), Some(Value::Int(vb))) => self.stack.push(Value::Int(va - vb)),
                        _ => return Err("Invalid types for Sub".to_string()),
                    }
                }
                Opcode::Mul => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Some(Value::Int(va)), Some(Value::Int(vb))) => self.stack.push(Value::Int(va * vb)),
                        _ => return Err("Invalid types for Mul".to_string()),
                    }
                }
                Opcode::Div => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Some(Value::Int(va)), Some(Value::Int(vb))) => {
                            if vb == 0 { return Err("Division by zero".to_string()); }
                            self.stack.push(Value::Int(va / vb))
                        }
                        _ => return Err("Invalid types for Div".to_string()),
                    }
                }
                Opcode::Eq => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(Value::Bool(match (a, b) {
                        (Some(Value::Int(va)), Some(Value::Int(vb))) => va == vb,
                        (Some(Value::Bool(va)), Some(Value::Bool(vb))) => va == vb,
                        (Some(Value::Object(oa)), Some(Value::Object(ob))) => oa == ob,
                        (Some(Value::Unit), Some(Value::Unit)) => true,
                        _ => false,
                    }));
                }
                Opcode::Ne => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(Value::Bool(match (a, b) {
                        (Some(Value::Int(va)), Some(Value::Int(vb))) => va != vb,
                        (Some(Value::Bool(va)), Some(Value::Bool(vb))) => va != vb,
                        (Some(Value::Object(oa)), Some(Value::Object(ob))) => oa != ob,
                        (Some(Value::Unit), Some(Value::Unit)) => false,
                        _ => true,
                    }));
                }
                Opcode::Lt => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if let (Some(Value::Int(va)), Some(Value::Int(vb))) = (a, b) {
                        self.stack.push(Value::Bool(va < vb));
                    } else { return Err("Invalid types for Lt".to_string()); }
                }
                Opcode::Gt => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if let (Some(Value::Int(va)), Some(Value::Int(vb))) = (a, b) {
                        self.stack.push(Value::Bool(va > vb));
                    } else { return Err("Invalid types for Gt".to_string()); }
                }
                Opcode::PushFloat(v) => self.stack.push(Value::Float(*v)),
                Opcode::PushStr(v) => {
                    let idx = self.heap.alloc(HeapObject::Str(v.clone()));
                    self.stack.push(Value::Object(idx));
                }
                Opcode::PushStruct(name, fields) => {
                    let mut field_vals = HashMap::new();
                    for field_name in fields.iter().rev() {
                        let val = self.stack.pop().ok_or("Stack underflow in PushStruct")?;
                        field_vals.insert(field_name.clone(), val);
                    }
                    let idx = self.heap.alloc(HeapObject::Struct(name.clone(), field_vals));
                    self.stack.push(Value::Object(idx));
                }
                Opcode::LoadField(name) => {
                    let val = self.stack.pop().ok_or("Stack underflow in LoadField")?;
                    if let Value::Object(idx) = val {
                        if let Some(HeapObject::Struct(_, fields)) = self.heap.get(idx) {
                            let f_val = fields.get(name).ok_or_else(|| format!("Field {} not found", name))?;
                            self.stack.push(f_val.clone());
                        } else {
                            return Err(format!("Cannot load field {} from non-struct object", name));
                        }
                    } else {
                        return Err(format!("Cannot load field {} from non-object value {:?}", name, val));
                    }
                }
                Opcode::Jmp(label) => {
                    frame.current_block_idx = func.blocks.iter().position(|b| &b.label == label).ok_or("Invalid jump")?;
                    frame.current_instr_idx = 0;
                }
                Opcode::JmpIfFalse(label) => {
                    let cond = self.stack.pop();
                    if let Some(Value::Bool(false)) = cond {
                        frame.current_block_idx = func.blocks.iter().position(|b| &b.label == label).ok_or("Invalid jump")?;
                        frame.current_instr_idx = 0;
                    }
                }
                Opcode::Call(name, arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..*arg_count {
                        args.push(self.stack.pop().ok_or("Stack underflow in Call")?);
                    }
                    args.reverse();

                    let target_func = self.functions.get(name).ok_or_else(|| format!("Function {} not found", name))?;
                    let mut locals = HashMap::new();
                    for (param, val) in target_func.params.iter().zip(args.into_iter()) {
                        locals.insert(param.clone(), val);
                    }

                    self.call_stack.push(Frame {
                        func_name: name.clone(),
                        current_block_idx: 0,
                        current_instr_idx: 0,
                        locals,
                    });
                }
                Opcode::Ret => {
                    self.call_stack.pop();
                    if self.call_stack.is_empty() {
                        return Ok(self.stack.pop().unwrap_or(Value::Unit));
                    }
                }
                _ => {}
            }
        }
        Ok(Value::Unit)
    }
}
