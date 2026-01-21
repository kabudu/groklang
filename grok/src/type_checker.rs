// grok/src/type_checker.rs
use crate::ast::{AstNode, Type};
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeEnv {
    vars: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, ty: Type) {
        self.vars.insert(name.to_string(), ty);
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
}

pub struct TypeChecker {
    env: TypeEnv,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
        }
    }

    pub fn check(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.check(node)?;
                }
                Ok(())
            }
            AstNode::Function {
                name,
                params,
                body,
                return_type,
            } => {
                // Add params to env
                for param in params {
                    if let Some(ty) = &param.ty {
                        self.env.insert(&param.name, ty.clone());
                    }
                }
                self.check(body)?;
                Ok(())
            }
            // Add more cases
            _ => Ok(()),
        }
    }
}
