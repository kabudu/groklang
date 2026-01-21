// grok/src/type_checker.rs
use crate::ast::{AstNode, Type, MatchArm};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub left: Type,
    pub right: Type,
}

pub struct TypeEnv {
    bindings: HashMap<String, Type>,
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn extend(self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(self)),
        }
    }

    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        if let Some(ty) = self.bindings.get(name) {
            Some(ty.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

pub struct TypeChecker {
    constraints: Vec<Constraint>,
    type_var_counter: usize,
    global_types: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            type_var_counter: 0,
            global_types: HashMap::new(),
        }
    }

    fn fresh_type_var(&mut self) -> Type {
        let name = format!("T{}", self.type_var_counter);
        self.type_var_counter += 1;
        Type::Variable(name)
    }

    pub fn check(&mut self, ast: &AstNode) -> Result<HashMap<String, Type>, String> {
        self.constraints.clear();
        self.global_types.clear();
        let mut env = TypeEnv::new();
        
        // Pass 1: Collect definitions
        self.collect_definitions(ast)?;
        
        // Pass 2: Collect constraints
        self.collect(ast, &mut env)?;

        let mut substitution = HashMap::new();
        self.unify(&mut substitution)?;

        Ok(substitution)
    }

    fn collect_definitions(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.collect_definitions(node)?;
                }
            }
            AstNode::StructDef { name, fields, .. } => {
                let ty = Type::Struct(name.clone(), fields.clone());
                self.global_types.insert(name.clone(), ty);
            }
            AstNode::EnumDef { name, .. } => {
                let ty = Type::Primitive(name.clone());
                self.global_types.insert(name.clone(), ty);
            }
            _ => {}
        }
        Ok(())
    }

    fn collect(&mut self, ast: &AstNode, env: &mut TypeEnv) -> Result<Type, String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.collect(node, env)?;
                }
                Ok(Type::Unit)
            }
            AstNode::FunctionDef { name, params, body, return_type, .. } => {
                let mut param_types = Vec::new();
                for param in params {
                    let ty = param.ty.clone().unwrap_or_else(|| self.fresh_type_var());
                    param_types.push(ty.clone());
                    env.bind(param.name.clone(), ty);
                }
                
                let body_type = self.collect(body, env)?;
                if let Some(ret_ty) = return_type {
                    self.constraints.push(Constraint {
                        left: body_type.clone(),
                        right: ret_ty.clone(),
                    });
                }
                
                let func_type = Type::Function(param_types, Box::new(body_type));
                env.bind(name.clone(), func_type.clone());
                Ok(func_type)
            }
            AstNode::StructDef { name, fields, .. } => {
                let ty = Type::Struct(name.clone(), fields.clone());
                self.global_types.insert(name.clone(), ty);
                Ok(Type::Unit)
            }
            AstNode::EnumDef { name, .. } => {
                // Simplified enum handling
                let ty = Type::Primitive(name.clone()); 
                self.global_types.insert(name.clone(), ty);
                Ok(Type::Unit)
            }
            AstNode::LetStmt { name, mutable: _, ty, expr, .. } => {
                let expr_type = self.collect(expr, env)?;
                if let Some(declared_ty) = ty {
                    self.constraints.push(Constraint {
                        left: expr_type.clone(),
                        right: declared_ty.clone(),
                    });
                }
                env.bind(name.clone(), expr_type.clone());
                Ok(expr_type)
            }
            AstNode::Block(stmts) => {
                let mut last_type = Type::Unit;
                for stmt in stmts {
                    last_type = self.collect(stmt, env)?;
                }
                Ok(last_type)
            }
            AstNode::IntLiteral(_, _) => Ok(Type::Primitive("i32".to_string())),
            AstNode::FloatLiteral(_, _) => Ok(Type::Primitive("f64".to_string())),
            AstNode::StringLiteral(_, _) => Ok(Type::Primitive("str".to_string())),
            AstNode::BoolLiteral(_, _) => Ok(Type::Primitive("bool".to_string())),
            AstNode::Identifier(name, _) => {
                env.lookup(name).ok_or_else(|| format!("Undefined variable: {}", name))
            }
            AstNode::BinaryOp { left, op, right, .. } => {
                let l_ty = self.collect(left, env)?;
                let r_ty = self.collect(right, env)?;
                
                if ["+", "-", "*", "/"].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty.clone(),
                        right: r_ty,
                    });
                    Ok(l_ty)
                } else if ["==", "!=", "<", ">", "<=", ">="].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty,
                        right: r_ty,
                    });
                    Ok(Type::Primitive("bool".to_string()))
                } else {
                    Ok(Type::Unit)
                }
            }
            AstNode::IfExpr { condition, then_body, else_body, .. } => {
                let cond_ty = self.collect(condition, env)?;
                self.constraints.push(Constraint {
                    left: cond_ty,
                    right: Type::Primitive("bool".to_string()),
                });
                
                let then_ty = self.collect(then_body, env)?;
                if let Some(else_b) = else_body {
                    let else_ty = self.collect(else_b, env)?;
                    self.constraints.push(Constraint {
                        left: then_ty.clone(),
                        right: else_ty,
                    });
                    Ok(then_ty)
                } else {
                    Ok(Type::Unit)
                }
            }
            AstNode::MatchExpr { scrutinee, arms, .. } => {
                let s_ty = self.collect(scrutinee, env)?;
                let res_ty = self.fresh_type_var();
                for arm in arms {
                    let p_ty = self.collect_pattern(&arm.pattern, env)?;
                    self.constraints.push(Constraint {
                        left: s_ty.clone(),
                        right: p_ty,
                    });
                    if let Some(guard) = &arm.guard {
                        let g_ty = self.collect(guard, env)?;
                        self.constraints.push(Constraint {
                            left: g_ty,
                            right: Type::Unit, // or bool
                        });
                    }
                    let b_ty = self.collect(&arm.body, env)?;
                    self.constraints.push(Constraint {
                        left: res_ty.clone(),
                        right: b_ty,
                    });
                }
                Ok(res_ty)
            }
            AstNode::StructLiteral { name, fields, .. } => {
                let struct_def_ty = self.global_types.get(name)
                    .ok_or_else(|| format!("Undefined struct: {}", name))?.clone();
                
                if let Type::Struct(_, def_fields) = struct_def_ty {
                    for (f_name, f_expr) in fields {
                        let f_expr_ty = self.collect(f_expr, env)?;
                        let def_f_ty = def_fields.iter().find(|(n, _)| n == f_name)
                            .map(|(_, t)| t)
                            .ok_or_else(|| format!("Unknown field {} in struct {}", f_name, name))?;
                        
                        self.constraints.push(Constraint {
                            left: f_expr_ty,
                            right: def_f_ty.clone(),
                        });
                    }
                    Ok(Type::Struct(name.clone(), def_fields))
                } else {
                    Err(format!("{} is not a struct", name))
                }
            }
            AstNode::MemberAccess { object, member, .. } => {
                let obj_ty = self.collect(object, env)?;
                match obj_ty {
                    Type::Struct(name, fields) => {
                        let field_ty = fields.iter().find(|(n, _)| n == member)
                            .map(|(_, t)| t.clone())
                            .ok_or_else(|| format!("Struct {} has no member {}", name, member))?;
                        Ok(field_ty)
                    }
                    _ => Err(format!("Cannot access member {} on non-struct type {:?}", member, obj_ty)),
                }
            }
            _ => Ok(Type::Unit),
        }
    }

    fn collect_pattern(&mut self, pattern: &crate::ast::Pattern, env: &mut TypeEnv) -> Result<Type, String> {
        match pattern {
            crate::ast::Pattern::Identifier(name) => {
                let ty = self.fresh_type_var();
                env.bind(name.clone(), ty.clone());
                Ok(ty)
            }
            crate::ast::Pattern::IntLiteral(_) => Ok(Type::Primitive("i32".to_string())),
            crate::ast::Pattern::BoolLiteral(_) => Ok(Type::Primitive("bool".to_string())),
            crate::ast::Pattern::Underscore => Ok(self.fresh_type_var()),
            _ => Ok(Type::Unit),
        }
    }

    fn unify(&self, substitution: &mut HashMap<String, Type>) -> Result<(), String> {
        let mut constraints = self.constraints.clone();
        while let Some(constraint) = constraints.pop() {
            let left = self.apply_subst(&constraint.left, substitution);
            let right = self.apply_subst(&constraint.right, substitution);

            if left == right { continue; }

            match (left, right) {
                (Type::Variable(name), ty) | (ty, Type::Variable(name)) => {
                    if self.occurs_check(&name, &ty) {
                        return Err(format!("Recursive type detected for {}", name));
                    }
                    substitution.insert(name, ty);
                }
                (Type::Primitive(p1), Type::Primitive(p2)) if p1 == p2 => {}
                (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                    if p1.len() != p2.len() {
                        return Err("Param count mismatch".to_string());
                    }
                    for (a, b) in p1.into_iter().zip(p2.into_iter()) {
                        constraints.push(Constraint { left: a, right: b });
                    }
                    constraints.push(Constraint { left: *r1, right: *r2 });
                }
                (l, r) => return Err(format!("Type mismatch: {:?} vs {:?}", l, r)),
            }
        }
        Ok(())
    }

    fn apply_subst(&self, ty: &Type, substitution: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Variable(name) => {
                if let Some(subst) = substitution.get(name) {
                    self.apply_subst(subst, substitution)
                } else {
                    ty.clone()
                }
            }
            Type::Function(params, ret) => {
                let params = params.iter().map(|p| self.apply_subst(p, substitution)).collect();
                let ret = Box::new(self.apply_subst(ret, substitution));
                Type::Function(params, ret)
            }
            _ => ty.clone(),
        }
    }

    fn occurs_check(&self, var: &str, ty: &Type) -> bool {
        match ty {
            Type::Variable(name) => name == var,
            Type::Function(params, ret) => {
                params.iter().any(|p| self.occurs_check(var, p)) || self.occurs_check(var, ret)
            }
            _ => false,
        }
    }
}
