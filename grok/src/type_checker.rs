// grok/src/type_checker.rs
use crate::ast::{AstNode, MatchArm, Pattern, Type};
use std::collections::{HashMap, HashSet};

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
    struct_arities: HashMap<String, usize>,
    enum_variants: HashMap<String, Vec<String>>,
    trait_names: HashSet<String>,
    trait_methods: HashMap<String, HashMap<String, MethodSignature>>,
}

#[derive(Debug, Clone, PartialEq)]
struct MethodSignature {
    params: Vec<Option<Type>>,
    return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            type_var_counter: 0,
            global_types: HashMap::new(),
            struct_arities: HashMap::new(),
            enum_variants: HashMap::new(),
            trait_names: HashSet::new(),
            trait_methods: HashMap::new(),
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
        self.struct_arities.clear();
        self.enum_variants.clear();
        self.trait_names.clear();
        self.trait_methods.clear();
        let mut env = TypeEnv::new();

        // Pass 1: Collect definitions
        self.collect_definitions(ast)?;

        // Pass 1.5: Validate declared type annotations.
        self.validate_type_annotations(ast)?;

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
                // Parser currently does not expose generic params for type defs.
                self.struct_arities.insert(name.clone(), 0);
            }
            AstNode::EnumDef { name, variants, .. } => {
                let ty = Type::Primitive(name.clone());
                self.global_types.insert(name.clone(), ty);
                self.enum_variants.insert(
                    name.clone(),
                    variants
                        .iter()
                        .map(|(variant, _)| variant.clone())
                        .collect(),
                );
            }
            AstNode::TraitDef { name, methods, .. } => {
                if self.trait_names.contains(name) {
                    return Err(format!("Duplicate trait definition: {}", name));
                }
                self.trait_names.insert(name.clone());
                self.global_types
                    .insert(name.clone(), Type::Trait(name.clone()));

                let mut method_sigs = HashMap::new();
                for method in methods {
                    let (method_name, sig) = Self::method_signature(method)?;
                    if method_sigs.insert(method_name.clone(), sig).is_some() {
                        return Err(format!(
                            "Duplicate method '{}' in trait '{}'",
                            method_name, name
                        ));
                    }
                }
                self.trait_methods.insert(name.clone(), method_sigs);
            }
            AstNode::ImplBlock {
                trait_name,
                for_type,
                methods,
                ..
            } => {
                if !self.global_types.contains_key(for_type) {
                    return Err(format!("Unknown type in impl block: {}", for_type));
                }

                let mut impl_method_sigs = HashMap::new();
                for method in methods {
                    let (method_name, sig) = Self::method_signature(method)?;
                    if impl_method_sigs.insert(method_name.clone(), sig).is_some() {
                        return Err(format!(
                            "Duplicate method '{}' in impl for '{}'",
                            method_name, for_type
                        ));
                    }
                }

                if let Some(trait_name) = trait_name {
                    if !self.trait_names.contains(trait_name) {
                        return Err(format!("Unknown trait in impl block: {}", trait_name));
                    }
                    if let Some(required) = self.trait_methods.get(trait_name) {
                        let missing: Vec<String> = required
                            .iter()
                            .filter_map(|(method_name, _)| {
                                if impl_method_sigs.contains_key(method_name) {
                                    None
                                } else {
                                    Some(method_name.clone())
                                }
                            })
                            .collect();
                        if !missing.is_empty() {
                            return Err(format!(
                                "Impl of trait '{}' for '{}' is missing method(s): {}",
                                trait_name,
                                for_type,
                                missing.join(", ")
                            ));
                        }

                        for (method_name, trait_sig) in required {
                            if let Some(impl_sig) = impl_method_sigs.get(method_name) {
                                Self::validate_impl_method_signature(
                                    trait_name,
                                    for_type,
                                    method_name,
                                    trait_sig,
                                    impl_sig,
                                )?;
                            }
                        }
                    }
                }
            }
            AstNode::ActorDef { name, .. } => {
                let ty = Type::Actor(name.clone());
                self.global_types.insert(name.clone(), ty);
            }
            AstNode::FunctionDef {
                name,
                params,
                return_type,
                ..
            } => {
                let p_tys = params
                    .iter()
                    .enumerate()
                    .map(|(idx, p)| {
                        p.ty.clone()
                            .unwrap_or_else(|| Type::Variable(format!("{}_p{}", name, idx)))
                    })
                    .collect();
                let r_ty = return_type.clone().unwrap_or(Type::Unit);
                self.global_types
                    .insert(name.clone(), Type::Function(p_tys, Box::new(r_ty)));
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_type_annotations(&self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) | AstNode::Block(nodes) => {
                for node in nodes {
                    self.validate_type_annotations(node)?;
                }
            }
            AstNode::FunctionDef {
                params,
                return_type,
                body,
                ..
            } => {
                for p in params {
                    if let Some(ty) = &p.ty {
                        self.validate_type(ty)?;
                    }
                }
                if let Some(ret) = return_type {
                    self.validate_type(ret)?;
                }
                self.validate_type_annotations(body)?;
            }
            AstNode::StructDef { fields, .. } => {
                for (_, ty) in fields {
                    self.validate_type(ty)?;
                }
            }
            AstNode::EnumDef { variants, .. } => {
                for (_, payload_ty) in variants {
                    if let Some(ty) = payload_ty {
                        self.validate_type(ty)?;
                    }
                }
            }
            AstNode::TraitDef {
                name,
                bounds,
                methods,
                ..
            } => {
                for bound in bounds {
                    if bound == name {
                        return Err(format!("Trait '{}' cannot bound itself", name));
                    }
                    if !self.trait_names.contains(bound) {
                        return Err(format!(
                            "Unknown trait bound '{}' on trait '{}'",
                            bound, name
                        ));
                    }
                }
                for method in methods {
                    self.validate_type_annotations(method)?;
                }
            }
            AstNode::ImplBlock { methods, .. } => {
                for method in methods {
                    self.validate_type_annotations(method)?;
                }
            }
            AstNode::LetStmt { ty, expr, .. } => {
                if let Some(ty) = ty {
                    self.validate_type(ty)?;
                }
                self.validate_type_annotations(expr)?;
            }
            AstNode::IfExpr {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_type_annotations(condition)?;
                self.validate_type_annotations(then_body)?;
                if let Some(else_body) = else_body {
                    self.validate_type_annotations(else_body)?;
                }
            }
            AstNode::WhileLoop {
                condition, body, ..
            } => {
                self.validate_type_annotations(condition)?;
                self.validate_type_annotations(body)?;
            }
            AstNode::ForLoop { iterable, body, .. } => {
                self.validate_type_annotations(iterable)?;
                self.validate_type_annotations(body)?;
            }
            AstNode::MatchExpr {
                scrutinee, arms, ..
            } => {
                self.validate_type_annotations(scrutinee)?;
                for arm in arms {
                    if let Some(g) = &arm.guard {
                        self.validate_type_annotations(g)?;
                    }
                    self.validate_type_annotations(&arm.body)?;
                }
            }
            AstNode::FunctionCall { func, args, .. } => {
                self.validate_type_annotations(func)?;
                for arg in args {
                    self.validate_type_annotations(arg)?;
                }
            }
            AstNode::UnaryOp { operand, .. } => self.validate_type_annotations(operand)?,
            AstNode::BinaryOp { left, right, .. } => {
                self.validate_type_annotations(left)?;
                self.validate_type_annotations(right)?;
            }
            AstNode::MemberAccess { object, .. } => self.validate_type_annotations(object)?,
            AstNode::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.validate_type_annotations(expr)?;
                }
            }
            AstNode::Return { value, .. } => {
                if let Some(v) = value {
                    self.validate_type_annotations(v)?;
                }
            }
            AstNode::Spawn { args, .. } => {
                for (_, expr) in args {
                    self.validate_type_annotations(expr)?;
                }
            }
            AstNode::Send {
                target, message, ..
            } => {
                self.validate_type_annotations(target)?;
                self.validate_type_annotations(message)?;
            }
            AstNode::Receive { arms, .. } => {
                for arm in arms {
                    if let Some(g) = &arm.guard {
                        self.validate_type_annotations(g)?;
                    }
                    self.validate_type_annotations(&arm.body)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn builtin_generic_arity(name: &str) -> Option<usize> {
        match name {
            "Vec" | "Option" | "HashSet" => Some(1),
            "Result" | "HashMap" => Some(2),
            _ => None,
        }
    }

    fn method_signature(method: &AstNode) -> Result<(String, MethodSignature), String> {
        match method {
            AstNode::FunctionDef {
                name,
                params,
                return_type,
                ..
            } => Ok((
                name.clone(),
                MethodSignature {
                    params: params.iter().map(|p| p.ty.clone()).collect(),
                    return_type: return_type.clone(),
                },
            )),
            _ => Err("Impl/trait methods must be function definitions".to_string()),
        }
    }

    fn validate_impl_method_signature(
        trait_name: &str,
        for_type: &str,
        method_name: &str,
        trait_sig: &MethodSignature,
        impl_sig: &MethodSignature,
    ) -> Result<(), String> {
        if trait_sig.params.len() != impl_sig.params.len() {
            return Err(format!(
                "Impl method '{}' for trait '{}' on '{}' has wrong arity: expected {}, got {}",
                method_name,
                trait_name,
                for_type,
                trait_sig.params.len(),
                impl_sig.params.len()
            ));
        }

        for (idx, (trait_param, impl_param)) in trait_sig
            .params
            .iter()
            .zip(impl_sig.params.iter())
            .enumerate()
        {
            if let Some(expected) = trait_param {
                if impl_param.as_ref() != Some(expected) {
                    return Err(format!(
                        "Impl method '{}' for trait '{}' on '{}' has incompatible type for parameter {}: expected {:?}, got {:?}",
                        method_name,
                        trait_name,
                        for_type,
                        idx + 1,
                        expected,
                        impl_param
                    ));
                }
            }
        }

        if let Some(expected_ret) = &trait_sig.return_type {
            if impl_sig.return_type.as_ref() != Some(expected_ret) {
                return Err(format!(
                    "Impl method '{}' for trait '{}' on '{}' has incompatible return type: expected {:?}, got {:?}",
                    method_name, trait_name, for_type, expected_ret, impl_sig.return_type
                ));
            }
        }

        Ok(())
    }

    fn validate_type(&self, ty: &Type) -> Result<(), String> {
        match ty {
            Type::Primitive(_) | Type::Variable(_) | Type::Unit | Type::Actor(_) => Ok(()),
            Type::Trait(name) => {
                if self.trait_names.contains(name) {
                    Ok(())
                } else {
                    Err(format!("Unknown trait type: {}", name))
                }
            }
            Type::Reference(inner, _) => self.validate_type(inner),
            Type::Function(params, ret) => {
                for p in params {
                    self.validate_type(p)?;
                }
                self.validate_type(ret)
            }
            Type::Struct(name, fields) => {
                if !self.global_types.contains_key(name) {
                    return Err(format!("Unknown struct type: {}", name));
                }
                for (_, field_ty) in fields {
                    self.validate_type(field_ty)?;
                }
                Ok(())
            }
            Type::Generic(name, args) => {
                for arg in args {
                    self.validate_type(arg)?;
                }

                if let Some(expected) = Self::builtin_generic_arity(name) {
                    if args.len() != expected {
                        return Err(format!(
                            "Generic type {} expects {} argument(s), got {}",
                            name,
                            expected,
                            args.len()
                        ));
                    }
                    return Ok(());
                }

                if let Some(expected) = self.struct_arities.get(name) {
                    if args.len() != *expected {
                        return Err(format!(
                            "Generic type {} expects {} argument(s), got {}",
                            name,
                            expected,
                            args.len()
                        ));
                    }
                    return Ok(());
                }

                Err(format!("Unknown generic type constructor: {}", name))
            }
        }
    }

    fn collect(&mut self, ast: &AstNode, env: &mut TypeEnv) -> Result<Type, String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.collect(node, env)?;
                }
                Ok(Type::Unit)
            }
            AstNode::FunctionDef {
                name,
                params,
                body,
                return_type,
                ..
            } => {
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
            AstNode::LetStmt {
                name,
                mutable: _,
                ty,
                expr,
                ..
            } => {
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
            AstNode::Identifier(name, span) => env
                .lookup(name)
                .or_else(|| self.global_types.get(name).cloned())
                .ok_or_else(|| {
                    format!(
                        "Undefined variable '{}' at line {} col {}",
                        name, span.line, span.col
                    )
                }),
            AstNode::FunctionCall { func, args, .. } => {
                let f_ty = self.collect(func, env)?;
                let res_ty = self.fresh_type_var();
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.collect(arg, env)?);
                }
                self.constraints.push(Constraint {
                    left: f_ty,
                    right: Type::Function(arg_types, Box::new(res_ty.clone())),
                });
                Ok(res_ty)
            }
            AstNode::BinaryOp {
                left, op, right, ..
            } => {
                let l_ty = self.collect(left, env)?;
                let r_ty = self.collect(right, env)?;

                if ["+", "-", "*", "/"].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty.clone(),
                        right: r_ty,
                    });
                    Ok(l_ty)
                } else if ["%", "&", "|", "^", "<<", ">>"].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty.clone(),
                        right: r_ty,
                    });
                    Ok(l_ty)
                } else if ["&&", "||"].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty,
                        right: Type::Primitive("bool".to_string()),
                    });
                    self.constraints.push(Constraint {
                        left: r_ty,
                        right: Type::Primitive("bool".to_string()),
                    });
                    Ok(Type::Primitive("bool".to_string()))
                } else if ["==", "!=", "<", ">", "<=", ">="].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty,
                        right: r_ty,
                    });
                    Ok(Type::Primitive("bool".to_string()))
                } else if ["="].contains(&op.as_str()) {
                    self.constraints.push(Constraint {
                        left: l_ty.clone(),
                        right: r_ty,
                    });
                    Ok(l_ty)
                } else if ["+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="]
                    .contains(&op.as_str())
                {
                    self.constraints.push(Constraint {
                        left: l_ty.clone(),
                        right: r_ty,
                    });
                    Ok(l_ty)
                } else {
                    Ok(Type::Unit)
                }
            }
            AstNode::IfExpr {
                condition,
                then_body,
                else_body,
                ..
            } => {
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
            AstNode::Return { value, .. } => {
                let ty = if let Some(v) = value {
                    self.collect(v, env)?
                } else {
                    Type::Unit
                };
                Ok(ty)
            }
            AstNode::Break { .. } | AstNode::Continue { .. } => Ok(Type::Unit),
            AstNode::MatchExpr {
                scrutinee, arms, ..
            } => {
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
                            right: Type::Primitive("bool".to_string()),
                        });
                    }
                    let b_ty = self.collect(&arm.body, env)?;
                    self.constraints.push(Constraint {
                        left: res_ty.clone(),
                        right: b_ty,
                    });
                }
                self.ensure_match_exhaustive(&s_ty, arms)?;
                Ok(res_ty)
            }
            AstNode::StructLiteral {
                name, fields, span, ..
            } => {
                let struct_def_ty = self
                    .global_types
                    .get(name)
                    .ok_or_else(|| {
                        format!(
                            "Undefined struct '{}' at line {} col {}",
                            name, span.line, span.col
                        )
                    })?
                    .clone();

                if let Type::Struct(_, def_fields) = struct_def_ty {
                    let mut provided = HashSet::new();
                    for (f_name, f_expr) in fields {
                        if !provided.insert(f_name.clone()) {
                            return Err(format!(
                                "Duplicate field '{}' in struct literal '{}' at line {} col {}",
                                f_name, name, span.line, span.col
                            ));
                        }
                        let f_expr_ty = self.collect(f_expr, env)?;
                        let def_f_ty = def_fields
                            .iter()
                            .find(|(n, _)| n == f_name)
                            .map(|(_, t)| t)
                            .ok_or_else(|| {
                                format!(
                                    "Unknown field '{}' in struct '{}' at line {} col {}",
                                    f_name, name, span.line, span.col
                                )
                            })?;

                        self.constraints.push(Constraint {
                            left: f_expr_ty,
                            right: def_f_ty.clone(),
                        });
                    }

                    let missing: Vec<String> = def_fields
                        .iter()
                        .filter_map(|(field_name, _)| {
                            if provided.contains(field_name) {
                                None
                            } else {
                                Some(field_name.clone())
                            }
                        })
                        .collect();
                    if !missing.is_empty() {
                        return Err(format!(
                            "Missing field(s) {} in struct literal '{}' at line {} col {}",
                            missing.join(", "),
                            name,
                            span.line,
                            span.col
                        ));
                    }
                    Ok(Type::Struct(name.clone(), def_fields))
                } else {
                    Err(format!(
                        "'{}' is not a struct at line {} col {}",
                        name, span.line, span.col
                    ))
                }
            }
            AstNode::MemberAccess {
                object,
                member,
                span,
                ..
            } => {
                let obj_ty = self.collect(object, env)?;
                match obj_ty {
                    Type::Struct(name, fields) => {
                        let field_ty = fields
                            .iter()
                            .find(|(n, _)| n == member)
                            .map(|(_, t)| t.clone())
                            .ok_or_else(|| {
                                format!(
                                    "Struct '{}' has no member '{}' at line {} col {}",
                                    name, member, span.line, span.col
                                )
                            })?;
                        Ok(field_ty)
                    }
                    _ => Err(format!(
                        "Cannot access member '{}' on non-struct type {:?} at line {} col {}",
                        member, obj_ty, span.line, span.col
                    )),
                }
            }
            AstNode::ActorDef { name, .. } => {
                let ty = Type::Actor(name.clone());
                self.global_types.insert(name.clone(), ty.clone());
                Ok(ty)
            }
            AstNode::Spawn {
                actor, args, span, ..
            } => {
                if !self.global_types.contains_key(actor) {
                    return Err(format!(
                        "Undefined actor '{}' at line {} col {}",
                        actor, span.line, span.col
                    ));
                }
                for (_, expr) in args {
                    self.collect(expr, env)?;
                }
                Ok(Type::Actor(actor.clone()))
            }
            AstNode::Send {
                target,
                message,
                span,
                ..
            } => {
                let t_ty = self.collect(target, env)?;
                let _m_ty = self.collect(message, env)?;

                match t_ty {
                    Type::Actor(_) => Ok(Type::Unit),
                    Type::Variable(_) => Ok(Type::Unit), // Optimistic
                    _ => Err(format!(
                        "Send target must be an actor, got {:?} at line {} col {}",
                        t_ty, span.line, span.col
                    )),
                }
            }
            AstNode::Receive { arms, .. } => {
                let res_ty = self.fresh_type_var();
                for arm in arms {
                    let _p_ty = self.collect_pattern(&arm.pattern, env)?;
                    if let Some(guard) = &arm.guard {
                        let g_ty = self.collect(guard, env)?;
                        self.constraints.push(Constraint {
                            left: g_ty,
                            right: Type::Primitive("bool".to_string()),
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
            _ => Ok(Type::Unit),
        }
    }

    fn collect_pattern(
        &mut self,
        pattern: &crate::ast::Pattern,
        env: &mut TypeEnv,
    ) -> Result<Type, String> {
        match pattern {
            crate::ast::Pattern::Or(patterns) => {
                if patterns.is_empty() {
                    return Ok(self.fresh_type_var());
                }
                let first_ty = self.collect_pattern(&patterns[0], env)?;
                for pat in &patterns[1..] {
                    let ty = self.collect_pattern(pat, env)?;
                    self.constraints.push(Constraint {
                        left: first_ty.clone(),
                        right: ty,
                    });
                }
                Ok(first_ty)
            }
            crate::ast::Pattern::Identifier(name) => {
                let ty = self.fresh_type_var();
                env.bind(name.clone(), ty.clone());
                Ok(ty)
            }
            crate::ast::Pattern::IntLiteral(_) => Ok(Type::Primitive("i32".to_string())),
            crate::ast::Pattern::FloatLiteral(_) => Ok(Type::Primitive("f64".to_string())),
            crate::ast::Pattern::StringLiteral(_) => Ok(Type::Primitive("str".to_string())),
            crate::ast::Pattern::BoolLiteral(_) => Ok(Type::Primitive("bool".to_string())),
            crate::ast::Pattern::Underscore => Ok(self.fresh_type_var()),
            crate::ast::Pattern::Tuple(items) => {
                for item in items {
                    self.collect_pattern(item, env)?;
                }
                Ok(self.fresh_type_var())
            }
            crate::ast::Pattern::Struct(name, fields) => {
                for (_, pat) in fields {
                    self.collect_pattern(pat, env)?;
                }
                Ok(Type::Struct(name.clone(), vec![]))
            }
            crate::ast::Pattern::Enum(enum_name, _variant, payload) => {
                if let Some(p) = payload {
                    self.collect_pattern(p, env)?;
                }
                Ok(Type::Primitive(enum_name.clone()))
            }
        }
    }

    fn ensure_match_exhaustive(
        &self,
        scrutinee_ty: &Type,
        arms: &[MatchArm],
    ) -> Result<(), String> {
        if scrutinee_ty == &Type::Primitive("bool".to_string()) {
            let mut has_true = false;
            let mut has_false = false;
            let mut has_wildcard = false;

            for arm in arms {
                let (t, f, w) = Self::bool_pattern_coverage(&arm.pattern);
                has_true |= t;
                has_false |= f;
                has_wildcard |= w;
            }

            if has_wildcard || (has_true && has_false) {
                return Ok(());
            }
            return Err(
                "Non-exhaustive match for bool: expected true and false branches (or _)"
                    .to_string(),
            );
        }

        if let Type::Primitive(enum_name) = scrutinee_ty {
            if let Some(variants) = self.enum_variants.get(enum_name) {
                let mut covered: HashMap<String, bool> =
                    variants.iter().map(|v| (v.clone(), false)).collect();
                let mut wildcard = false;
                for arm in arms {
                    for variant in Self::enum_covered_variants(enum_name, &arm.pattern) {
                        covered.insert(variant, true);
                    }
                    if Self::pattern_has_wildcard(&arm.pattern) {
                        wildcard = true;
                    }
                }

                if wildcard || covered.values().all(|v| *v) {
                    return Ok(());
                }
                let missing: Vec<String> = covered
                    .iter()
                    .filter_map(|(v, is_covered)| if !is_covered { Some(v.clone()) } else { None })
                    .collect();
                return Err(format!(
                    "Non-exhaustive match for enum {}: missing variant(s): {}",
                    enum_name,
                    missing.join(", ")
                ));
            }
        }

        Ok(())
    }

    fn bool_pattern_coverage(pattern: &Pattern) -> (bool, bool, bool) {
        match pattern {
            Pattern::BoolLiteral(true) => (true, false, false),
            Pattern::BoolLiteral(false) => (false, true, false),
            Pattern::Underscore | Pattern::Identifier(_) => (false, false, true),
            Pattern::Or(patterns) => patterns.iter().fold((false, false, false), |acc, p| {
                let (t, f, w) = Self::bool_pattern_coverage(p);
                (acc.0 || t, acc.1 || f, acc.2 || w)
            }),
            _ => (false, false, false),
        }
    }

    fn pattern_has_wildcard(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Underscore | Pattern::Identifier(_) => true,
            Pattern::Or(patterns) => patterns.iter().any(Self::pattern_has_wildcard),
            _ => false,
        }
    }

    fn enum_covered_variants(enum_name: &str, pattern: &Pattern) -> Vec<String> {
        match pattern {
            Pattern::Enum(pattern_enum_name, variant, _) if pattern_enum_name == enum_name => {
                vec![variant.clone()]
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .flat_map(|p| Self::enum_covered_variants(enum_name, p))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn unify(&self, substitution: &mut HashMap<String, Type>) -> Result<(), String> {
        let mut constraints = self.constraints.clone();
        while let Some(constraint) = constraints.pop() {
            let left = self.apply_subst(&constraint.left, substitution);
            let right = self.apply_subst(&constraint.right, substitution);

            if left == right {
                continue;
            }

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
                    constraints.push(Constraint {
                        left: *r1,
                        right: *r2,
                    });
                }
                (Type::Struct(n1, _), Type::Struct(n2, _)) if n1 == n2 => {}

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
                let params = params
                    .iter()
                    .map(|p| self.apply_subst(p, substitution))
                    .collect();
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
