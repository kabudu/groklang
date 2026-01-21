use crate::ast::{AstNode, Span};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowType {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct Borrow {
    pub var_name: String,
    pub borrow_type: BorrowType,
    pub span: Span,
}

pub struct BorrowChecker {
    // Current active borrows per variable. Tracks across scopes.
    borrow_stack: Vec<HashMap<String, Vec<Borrow>>>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            borrow_stack: vec![HashMap::new()],
        }
    }

    fn enter_scope(&mut self) {
        self.borrow_stack.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.borrow_stack.pop();
    }

    pub fn check(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.check(node)?;
                }
            }
            AstNode::FunctionDef { body, .. } => {
                self.enter_scope();
                self.check(body)?;
                self.exit_scope();
            }
            AstNode::Block(stmts) => {
                self.enter_scope();
                for stmt in stmts {
                    self.check(stmt)?;
                }
                self.exit_scope();
            }
            AstNode::LetStmt { expr, .. } => {
                self.check(expr)?;
            }
            AstNode::UnaryOp { op, operand, span } => {
                if op == "&" || op == "&mut" {
                    if let AstNode::Identifier(name, _) = &**operand {
                        let b_type = if op == "&mut" { BorrowType::Mutable } else { BorrowType::Immutable };
                        self.add_borrow(name.clone(), b_type, span.clone())?;
                    } else {
                        return Err(format!("Cannot borrow non-identifier at {:?}", span));
                    }
                }
                self.check(operand)?;
            }
            AstNode::BinaryOp { left, right, .. } => {
                self.check(left)?;
                self.check(right)?;
            }
            AstNode::IfExpr { condition, then_body, else_body, .. } => {
                self.check(condition)?;
                self.check(then_body)?;
                if let Some(e) = else_body {
                    self.check(e)?;
                }
            }
            // Add more nodes
            _ => {}
        }
        Ok(())
    }

    pub fn add_borrow(&mut self, var: String, b_type: BorrowType, span: Span) -> Result<(), String> {
        // Enforce exclusivity rules across ALL scopes for this variable
        let mut total_borrows = 0;
        let mut has_mutable = false;

        for scope in &self.borrow_stack {
            if let Some(existing) = scope.get(&var) {
                total_borrows += existing.len();
                if existing.iter().any(|b| b.borrow_type == BorrowType::Mutable) {
                    has_mutable = true;
                }
            }
        }

        if b_type == BorrowType::Mutable {
            if total_borrows > 0 {
                return Err(format!("Cannot mutably borrow '{}' at {:?} because it is already borrowed", var, span));
            }
        } else {
            if has_mutable {
                return Err(format!("Cannot immutably borrow '{}' at {:?} because it is mutably borrowed", var, span));
            }
        }
        
        // Add to current scope
        let current_scope = self.borrow_stack.last_mut().unwrap();
        current_scope.entry(var.clone()).or_insert(Vec::new()).push(Borrow { var_name: var, borrow_type: b_type, span });
        Ok(())
    }
}
