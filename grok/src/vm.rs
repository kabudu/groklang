// grok/src/vm.rs

use crate::ir::{IRFunction, IRInstruction};
use std::collections::HashMap;

pub struct VM {
    stack: Vec<i64>,
    variables: HashMap<String, i64>,
    functions: HashMap<String, IRFunction>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn load_program(&mut self, functions: &[IRFunction]) {
        for func in functions {
            self.functions.insert(func.name.clone(), func.clone());
        }
    }

    pub fn execute(&mut self, func_name: &str) -> Result<(), String> {
        let func = self.functions.get(func_name).ok_or("Function not found")?;
        for instr in &func.instructions {
            match instr {
                IRInstruction::Load(var) => {
                    if let Some(&val) = self.variables.get(var) {
                        self.stack.push(val);
                    }
                }
                IRInstruction::Store(var) => {
                    if let Some(val) = self.stack.pop() {
                        self.variables.insert(var.clone(), val);
                    }
                }
                IRInstruction::Add => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    self.stack.push(a + b);
                }
                IRInstruction::Return => break,
                _ => {} // Implement others
            }
        }
        Ok(())
    }
}
