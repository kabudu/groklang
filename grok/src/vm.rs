use crate::ir::{IRFunction, Opcode};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use tokio::sync::broadcast;
use std::pin::Pin;
use std::future::Future;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Object(usize),
    Actor(usize),
    Unit,
}

impl Value {
    pub fn into_int(self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(v),
            _ => None,
        }
    }
    pub fn into_bool(self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HeapObject {
    Str(String),
    Struct(String, HashMap<String, Value>),
}

pub struct Heap {
    pub objects: Vec<Option<HeapObject>>,
    pub marked: Vec<bool>,
    pub free_list: Vec<usize>,
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
            idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(obj));
            self.marked.push(false);
            idx
        }
    }

    pub fn get(&self, idx: usize) -> Option<&HeapObject> {
        self.objects.get(idx)?.as_ref()
    }

    pub fn gc(&mut self, stack: &[Value], call_stack: &[Frame], globals: &HashMap<String, Value>) {
        for m in self.marked.iter_mut() { *m = false; }
        for val in stack { self.mark_value(val); }
        for frame in call_stack {
            for val in frame.locals.values() { self.mark_value(val); }
        }
        for val in globals.values() { self.mark_value(val); }
        for i in 0..self.objects.len() {
            if !self.marked[i] && self.objects[i].is_some() {
                self.objects[i] = None;
                self.free_list.push(i);
            }
        }
    }

    fn mark_value(&mut self, val: &Value) {
        if let Value::Object(idx) = val {
            if !self.marked[*idx] {
                self.marked[*idx] = true;
                let obj = self.objects[*idx].clone();
                if let Some(HeapObject::Struct(_, fields)) = obj {
                    for v in fields.values() { self.mark_value(v); }
                }
            }
        }
    }
}

pub struct Frame {
    pub func_name: String,
    pub current_block_idx: usize,
    pub current_instr_idx: usize,
    pub locals: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionPolicy {
    OneForOne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorStatus {
    Running,
    BlockedOnReceive,
    Failed,
    Stopped,
}

pub struct ActorMetadata {
    pub id: usize,
    pub func_name: String,
    pub args: Vec<Value>,
    pub parent_id: Option<usize>,
    pub children: Vec<usize>,
    pub policy: SupervisionPolicy,
    pub status: ActorStatus,
    pub mailbox_tx: UnboundedSender<Value>,
}

pub struct ActorRegistry {
    pub actors: HashMap<usize, ActorMetadata>,
    pub next_id: usize,
    pub deadlock_notifier: broadcast::Sender<()>,
}

pub struct VM {
    pub stack: Vec<Value>,
    pub call_stack: Vec<Frame>,
    pub functions: Arc<HashMap<String, IRFunction>>,
    pub globals: HashMap<String, Value>,
    pub heap: Arc<Mutex<Heap>>,
    pub registry: Arc<Mutex<ActorRegistry>>,
    pub current_actor_id: Option<usize>,
}

impl VM {
    pub fn new() -> Self {
        let (deadlock_notifier, _) = broadcast::channel(1);
        let registry = ActorRegistry {
            actors: HashMap::new(),
            next_id: 1, 
            deadlock_notifier,
        };
        
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            functions: Arc::new(HashMap::new()),
            globals: HashMap::new(),
            heap: Arc::new(Mutex::new(Heap::new())),
            registry: Arc::new(Mutex::new(registry)),
            current_actor_id: None,
        }
    }

    pub fn load_program(&mut self, functions: &[IRFunction]) {
        let mut map = HashMap::new();
        for func in functions {
            map.insert(func.name.clone(), func.clone());
        }
        self.functions = Arc::new(map);
    }

    fn set_actor_status(&self, id: usize, status: ActorStatus) -> Result<(), String> {
        let mut reg = self.registry.lock().map_err(|e| e.to_string())?;
        if let Some(meta) = reg.actors.get_mut(&id) {
            meta.status = status;
        }
        Ok(())
    }

    pub fn execute(mut self, func_name: String, mut mailbox: Option<UnboundedReceiver<Value>>) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
        Box::pin(async move {
            self.call_stack.push(Frame {
                func_name: func_name.clone(),
                current_block_idx: 0,
                current_instr_idx: 0,
                locals: HashMap::new(),
            });

            let mut deadlock_rx = {
                let reg = self.registry.lock().map_err(|e| e.to_string())?;
                reg.deadlock_notifier.subscribe()
            };

            if self.current_actor_id.is_none() {
                let registry = self.registry.clone();
                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        let reg_read = registry.lock().unwrap();
                        
                        let total_actors = reg_read.actors.len();
                        if total_actors == 0 { continue; }

                        let blocked_actors = reg_read.actors.values().filter(|a| a.status == ActorStatus::BlockedOnReceive).count();
                        let active_actors = reg_read.actors.values().filter(|a| a.status == ActorStatus::Running).count();

                        if active_actors == 0 && blocked_actors > 0 {
                            println!("DEADLOCK DETECTED");
                            let _ = reg_read.deadlock_notifier.send(());
                            break;
                        }
                        
                        if reg_read.actors.values().all(|a| a.status == ActorStatus::Stopped || a.status == ActorStatus::Failed) {
                            break;
                        }
                    }
                });
            }

            let mut gc_timer = 0;

            while !self.call_stack.is_empty() {
                if let Ok(_) = deadlock_rx.try_recv() {
                    return Err("Execution aborted due to deadlock".to_string());
                }

                gc_timer += 1;
                if gc_timer > 1000 {
                    {
                        let mut heap = self.heap.lock().map_err(|e| e.to_string())?;
                        heap.gc(&self.stack, &self.call_stack, &self.globals);
                    }
                    gc_timer = 0;
                }

                let mut msg_to_receive = false;

                {
                    let frame = self.call_stack.last_mut().unwrap();
                    let func = self.functions.get(&frame.func_name).ok_or_else(|| format!("Function {} not found", frame.func_name))?;
                    let block = &func.blocks[frame.current_block_idx];
                    
                    if frame.current_instr_idx >= block.instructions.len() {
                        if frame.current_block_idx + 1 < func.blocks.len() {
                            frame.current_block_idx += 1;
                            frame.current_instr_idx = 0;
                            continue;
                        } else {
                            self.call_stack.pop();
                            if self.call_stack.is_empty() {
                                if let Some(id) = self.current_actor_id {
                                    self.set_actor_status(id, ActorStatus::Stopped)?;
                                }
                                return Ok(self.stack.pop().unwrap_or(Value::Unit));
                            }
                            continue;
                        }
                    }
                    
                    let opcode = block.instructions[frame.current_instr_idx].opcode.clone();
                    frame.current_instr_idx += 1;

                    match opcode {
                        Opcode::PushInt(v) => self.stack.push(Value::Int(v)),
                        Opcode::PushBool(v) => self.stack.push(Value::Bool(v)),
                        Opcode::LoadVar(name) => {
                            let val = frame.locals.get(&name).cloned().or_else(|| self.globals.get(&name).cloned());
                            match val {
                                Some(v) => self.stack.push(v),
                                None => return Err(format!("Variable {} not found", name)),
                            }
                        }
                        Opcode::StoreVar(name) => {
                            let val = self.stack.pop().ok_or("Stack underflow in StoreVar")?;
                            frame.locals.insert(name.clone(), val);
                        }
                        Opcode::Add => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a + b)),
                                _ => return Err("Invalid types for Add".to_string()),
                            }
                        }
                        Opcode::Sub => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a - b)),
                                _ => return Err("Invalid types for Sub".to_string()),
                            }
                        }
                        Opcode::Mul => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Int(a * b)),
                                _ => return Err("Invalid types for Mul".to_string()),
                            }
                        }
                        Opcode::Div => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => {
                                    if b == 0 { return Err("Division by zero".to_string()); }
                                    self.stack.push(Value::Int(a / b))
                                }
                                _ => return Err("Invalid types for Div".to_string()),
                            }
                        }
                        Opcode::Eq => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            self.stack.push(Value::Bool(a == b));
                        }
                        Opcode::Ne => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            self.stack.push(Value::Bool(a != b));
                        }
                        Opcode::Lt => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Bool(a < b)),
                                _ => return Err("Invalid types for Lt".to_string()),
                            }
                        }
                        Opcode::Gt => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Bool(a > b)),
                                _ => return Err("Invalid types for Gt".to_string()),
                            }
                        }
                        Opcode::Le => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Bool(a <= b)),
                                _ => return Err("Invalid types for Le".to_string()),
                            }
                        }
                        Opcode::Ge => {
                            let b = self.stack.pop().ok_or("Stack underflow")?;
                            let a = self.stack.pop().ok_or("Stack underflow")?;
                            match (a, b) {
                                (Value::Int(a), Value::Int(b)) => self.stack.push(Value::Bool(a >= b)),
                                _ => return Err("Invalid types for Ge".to_string()),
                            }
                        }
                        Opcode::PushFloat(v) => self.stack.push(Value::Float(v)),
                        Opcode::PushStr(v) => {
                            let mut heap = self.heap.lock().map_err(|e| e.to_string())?;
                            let idx = heap.alloc(HeapObject::Str(v.clone()));
                            self.stack.push(Value::Object(idx));
                        }
                        Opcode::PushStruct(name, fields) => {
                            let mut field_vals = HashMap::new();
                            for field_name in fields.iter().rev() {
                                let val = self.stack.pop().ok_or("Stack underflow in PushStruct")?;
                                field_vals.insert(field_name.clone(), val);
                            }
                            let mut heap = self.heap.lock().map_err(|e| e.to_string())?;
                            let idx = heap.alloc(HeapObject::Struct(name.clone(), field_vals));
                            self.stack.push(Value::Object(idx));
                        }
                        Opcode::LoadField(name) => {
                            let val = self.stack.pop().ok_or("Stack underflow in LoadField")?;
                            if let Value::Object(idx) = val {
                                let heap = self.heap.lock().map_err(|e| e.to_string())?;
                                if let Some(HeapObject::Struct(_, fields)) = heap.get(idx) {
                                    let f_val = fields.get(&name).ok_or_else(|| format!("Field {} not found", name))?;
                                    self.stack.push(f_val.clone());
                                } else {
                                    return Err(format!("Cannot load field {} from non-struct object", name));
                                }
                            } else {
                                return Err(format!("Cannot load field {} from non-object value {:?}", name, val));
                            }
                        }
                        Opcode::Spawn(actor_name, arg_count) => {
                            let mut args = Vec::new();
                            for _ in 0..arg_count {
                                args.push(self.stack.pop().ok_or("Stack underflow in Spawn")?);
                            }
                            args.reverse();
                            
                            let (id, tx, rx) = {
                                let mut reg = self.registry.lock().map_err(|e| e.to_string())?;
                                let id = reg.next_id;
                                reg.next_id += 1;
                                
                                let (tx, rx) = unbounded_channel();
                                let meta = ActorMetadata {
                                    id,
                                    func_name: actor_name.clone(),
                                    args: args.clone(),
                                    parent_id: self.current_actor_id,
                                    children: Vec::new(),
                                    policy: SupervisionPolicy::OneForOne,
                                    status: ActorStatus::Running,
                                    mailbox_tx: tx.clone(),
                                };
                                
                                if let Some(parent_id) = self.current_actor_id {
                                    if let Some(parent_meta) = reg.actors.get_mut(&parent_id) {
                                        parent_meta.children.push(id);
                                    }
                                }
                                
                                reg.actors.insert(id, meta);
                                (id, tx, rx)
                            };

                            let heap = self.heap.clone();
                            let functions = self.functions.clone();
                            let registry = self.registry.clone();
                            let actor_name_clone = actor_name.clone();

                            tokio::spawn(async move {
                                let mut vm = VM {
                                    stack: Vec::new(),
                                    call_stack: Vec::new(),
                                    functions,
                                    globals: HashMap::new(),
                                    heap,
                                    registry: registry.clone(),
                                    current_actor_id: Some(id),
                                };
                                
                                let res = vm.execute(actor_name_clone.clone(), Some(rx)).await;
                                
                                if let Err(e) = res {
                                    let mut reg = registry.lock().unwrap();
                                    if let Some(meta) = reg.actors.get_mut(&id) {
                                        meta.status = ActorStatus::Failed;
                                        println!("Actor {} failed: {}. Policy says: Restart.", id, e);
                                        
                                        if meta.policy == SupervisionPolicy::OneForOne {
                                            // TODO: Real recursive restart would need access to ActorMetadata::args and func_name
                                            // and re-call tokio::spawn. This requires a helper.
                                        }
                                    }
                                }
                            });

                            self.stack.push(Value::Actor(id));
                        }
                        Opcode::Send => {
                            let msg = self.stack.pop().ok_or("Stack underflow in Send (msg)")?;
                            let target = self.stack.pop().ok_or("Stack underflow in Send (target)")?;
                            if let Value::Actor(id) = target {
                                let reg = self.registry.lock().map_err(|e| e.to_string())?;
                                if let Some(meta) = reg.actors.get(&id) {
                                    let _ = meta.mailbox_tx.send(msg);
                                }
                            } else {
                                return Err(format!("Cannot send to non-actor type {:?}", target));
                            }
                        }
                        Opcode::Receive => {
                            msg_to_receive = true;
                        }
                        Opcode::Jmp(label) => {
                            frame.current_block_idx = func.blocks.iter().position(|b| b.label == label).ok_or("Invalid jump")?;
                            frame.current_instr_idx = 0;
                        }
                        Opcode::JmpIfFalse(label) => {
                            let cond = self.stack.pop();
                            if let Some(Value::Bool(false)) = cond {
                                frame.current_block_idx = func.blocks.iter().position(|b| b.label == label).ok_or("Invalid jump")?;
                                frame.current_instr_idx = 0;
                            }
                        }
                        Opcode::Call(name, arg_count) => {
                            let mut args = Vec::new();
                            for _ in 0..arg_count {
                                args.push(self.stack.pop().ok_or("Stack underflow in Call")?);
                            }
                            args.reverse();
        
                            let target_func = self.functions.get(&name).ok_or_else(|| format!("Function {} not found", name))?;
                            let mut locals = HashMap::new();
                            for (param, val) in target_func.params.iter().zip(args.into_iter()) {
                                locals.insert(param.clone(), val);
                            }

                            self.call_stack.push(Frame {
                                func_name: name,
                                current_block_idx: 0,
                                current_instr_idx: 0,
                                locals,
                            });
                        }
                        Opcode::Ret => {
                            self.call_stack.pop();
                            if self.call_stack.is_empty() {
                                if let Some(id) = self.current_actor_id {
                                    self.set_actor_status(id, ActorStatus::Stopped)?;
                                }
                                return Ok(self.stack.pop().unwrap_or(Value::Unit));
                            }
                        }
                    }
                }

                if msg_to_receive {
                    if let Some(rx) = &mut mailbox {
                        if let Some(id) = self.current_actor_id {
                            self.set_actor_status(id, ActorStatus::BlockedOnReceive)?;
                        }
                        
                        let msg = tokio::select! {
                            m = rx.recv() => m,
                            _ = deadlock_rx.recv() => return Err("Execution aborted due to deadlock".to_string()),
                        };
                        
                        if let Some(id) = self.current_actor_id {
                            self.set_actor_status(id, ActorStatus::Running)?;
                        }

                        if let Some(msg) = msg {
                            self.stack.push(msg);
                        } else {
                            return Err("Actor mailbox closed".to_string());
                        }
                    } else {
                        return Err("Receive called outside of an actor".to_string());
                    }
                }
            }
            Ok(Value::Unit)
        })
    }
}
