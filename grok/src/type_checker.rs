// grok/src/type_checker.rs
use crate::ast::{AstNode, MatchArm, Pattern, Type};
use crate::macro_expander::MacroExpander;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub left: Type,
    pub right: Type,
}

#[derive(Clone)]
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
    enum_variant_payloads: HashMap<String, HashMap<String, Option<Type>>>,
    trait_names: HashSet<String>,
    trait_bounds: HashMap<String, Vec<String>>,
    trait_methods: HashMap<String, HashMap<String, MethodSignature>>,
    trait_impls: HashSet<(String, String)>,
    generic_trait_impls: Vec<GenericTraitImpl>,
    function_where_bounds: HashMap<String, Vec<(String, Vec<String>)>>,
    module_symbols: HashMap<String, HashSet<String>>,
    module_children: HashMap<String, HashSet<String>>,
    module_decls: HashSet<String>,
    actor_message_types: HashMap<String, Type>,
    macro_names: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct MethodSignature {
    params: Vec<Option<Type>>,
    return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
struct GenericTraitImpl {
    trait_name: String,
    for_type_head: String,
    for_type_params: Vec<String>,
    for_type_param_positions: HashMap<String, Vec<usize>>,
    generic_bounds: HashMap<String, Vec<String>>,
}

impl TypeChecker {
    const RECURSIVE_DOMAIN_WILDCARD: &'static str = "*";

    fn format_span(span: &crate::ast::Span) -> String {
        format!(" at line {} col {}", span.line, span.col)
    }

    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            type_var_counter: 0,
            global_types: HashMap::new(),
            struct_arities: HashMap::new(),
            enum_variants: HashMap::new(),
            enum_variant_payloads: HashMap::new(),
            trait_names: HashSet::new(),
            trait_bounds: HashMap::new(),
            trait_methods: HashMap::new(),
            trait_impls: HashSet::new(),
            generic_trait_impls: Vec::new(),
            function_where_bounds: HashMap::new(),
            module_symbols: HashMap::new(),
            module_children: HashMap::new(),
            module_decls: HashSet::new(),
            actor_message_types: HashMap::new(),
            macro_names: HashSet::new(),
        }
    }

    fn register_builtin_types(&mut self) {
        for p in [
            "bool", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
            "u128", "usize", "f32", "f64", "char", "str", "String", "Vec", "Option",
            "Result", "HashMap", "HashSet",
        ] {
            self.global_types
                .entry(p.to_string())
                .or_insert_with(|| Type::Primitive(p.to_string()));
        }
    }

    fn fresh_type_var(&mut self) -> Type {
        let name = format!("T{}", self.type_var_counter);
        self.type_var_counter += 1;
        Type::Variable(name)
    }

    pub fn check(&mut self, ast: &AstNode) -> Result<HashMap<String, Type>, String> {
        let mut expander = MacroExpander::new();
        let expanded_ast = expander.expand(ast.clone());

        self.constraints.clear();
        self.global_types.clear();
        self.struct_arities.clear();
        self.enum_variants.clear();
        self.enum_variant_payloads.clear();
        self.trait_names.clear();
        self.trait_bounds.clear();
        self.trait_methods.clear();
        self.trait_impls.clear();
        self.generic_trait_impls.clear();
        self.function_where_bounds.clear();
        self.module_symbols.clear();
        self.module_children.clear();
        self.module_decls.clear();
        self.actor_message_types.clear();
        self.macro_names.clear();
        self.register_builtin_types();
        let mut env = TypeEnv::new();

        // Pass 1: Collect definitions
        self.collect_definitions(&expanded_ast)?;
        self.collect_module_index(&expanded_ast, &[]);

        // Pass 1.5: Validate declared type annotations.
        self.validate_type_annotations(&expanded_ast)?;

        // Pass 1.75: Materialize internal use imports/aliases into checker globals.
        self.collect_use_bindings(&expanded_ast)?;

        // Pass 2: Collect constraints
        self.collect(&expanded_ast, &mut env)?;

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
            AstNode::ModuleDecl { .. } => {}
            AstNode::ModuleDef { items, .. } => {
                for node in items {
                    self.collect_definitions(node)?;
                }
            }
            AstNode::UseDecl { .. } => {}
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
                self.enum_variant_payloads.insert(
                    name.clone(),
                    variants
                        .iter()
                        .map(|(variant, payload)| (variant.clone(), payload.clone()))
                        .collect(),
                );
            }
            AstNode::TraitDef {
                name,
                bounds,
                methods,
                span,
                ..
            } => {
                if self.trait_names.contains(name) {
                    return Err(format!(
                        "Duplicate trait definition: {}{}",
                        name,
                        Self::format_span(span)
                    ));
                }
                self.trait_names.insert(name.clone());
                self.trait_bounds.insert(name.clone(), bounds.clone());
                self.global_types
                    .insert(name.clone(), Type::Trait(name.clone()));

                let mut method_sigs = HashMap::new();
                for method in methods {
                    let (method_name, sig) = Self::method_signature(method)?;
                    if method_sigs.insert(method_name.clone(), sig).is_some() {
                        return Err(format!(
                            "Duplicate method '{}' in trait '{}'{}",
                            method_name,
                            name,
                            Self::format_span(span)
                        ));
                    }
                }
                self.trait_methods.insert(name.clone(), method_sigs);
            }
            AstNode::ImplBlock {
                trait_name,
                for_type,
                for_type_params,
                generic_bounds,
                methods,
                span,
                ..
            } => {
                if !self.global_types.contains_key(for_type) {
                    return Err(format!(
                        "Unknown type in impl block: {}{}",
                        for_type,
                        Self::format_span(span)
                    ));
                }

                let for_type_params_set: HashSet<String> = for_type_params.iter().cloned().collect();
                for (bound_param, _) in generic_bounds {
                    if !for_type_params_set.contains(bound_param) {
                        return Err(format!(
                            "Impl generic bound references '{}' not present in impl type parameter list{}",
                            bound_param,
                            Self::format_span(span)
                        ));
                    }
                }

                for (_, bounds) in generic_bounds {
                    for bound_trait in bounds {
                        if !self.trait_names.contains(bound_trait) {
                            return Err(format!(
                                "Unknown trait bound '{}' in impl for '{}'{}",
                                bound_trait,
                                for_type,
                                Self::format_span(span)
                            ));
                        }
                    }
                }

                let mut impl_method_sigs = HashMap::new();
                for method in methods {
                    let (method_name, sig) = Self::method_signature(method)?;
                    if impl_method_sigs.insert(method_name.clone(), sig).is_some() {
                        return Err(format!(
                            "Duplicate method '{}' in impl for '{}'{}",
                            method_name,
                            for_type,
                            Self::format_span(span)
                        ));
                    }
                }

                if let Some(trait_name) = trait_name {
                    if !self.trait_names.contains(trait_name) {
                        return Err(format!(
                            "Unknown trait in impl block: {}{}",
                            trait_name,
                            Self::format_span(span)
                        ));
                    }
                    if for_type_params.is_empty() && generic_bounds.is_empty() {
                        self.trait_impls
                            .insert((trait_name.clone(), for_type.clone()));
                    } else {
                self.generic_trait_impls.push(GenericTraitImpl {
                            trait_name: trait_name.clone(),
                            for_type_head: for_type.clone(),
                            for_type_params: for_type_params.clone(),
                            for_type_param_positions: Self::for_type_param_positions(for_type_params),
                            generic_bounds: generic_bounds
                                .iter()
                                .map(|(p, b)| (p.clone(), b.clone()))
                                .collect(),
                        });
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
                                "Impl of trait '{}' for '{}' is missing method(s): {}{}",
                                trait_name,
                                for_type,
                                missing.join(", "),
                                Self::format_span(span)
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
            AstNode::MacroDef { name, .. } => {
                self.macro_names.insert(name.clone());
            }
            AstNode::FunctionDef {
                name,
                params,
                return_type,
                decorators,
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
                self.function_where_bounds
                    .insert(name.clone(), Self::parse_where_bounds(decorators));
            }
            _ => {}
        }
        Ok(())
    }

    fn parse_where_bounds(decorators: &[String]) -> Vec<(String, Vec<String>)> {
        let mut out = Vec::new();
        for d in decorators {
            if let Some(payload) = d.strip_prefix("__where:") {
                let mut parts = payload.splitn(2, ':');
                let type_var = parts.next().unwrap_or_default();
                let traits_raw = parts.next().unwrap_or_default();
                if type_var.is_empty() || traits_raw.is_empty() {
                    continue;
                }
                out.push((
                    type_var.to_string(),
                    traits_raw.split('+').map(|s| s.to_string()).collect(),
                ));
            }
        }
        out
    }

    fn validate_type_annotations(&self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) | AstNode::Block(nodes) => {
                for node in nodes {
                    self.validate_type_annotations(node)?;
                }
            }
            AstNode::ModuleDecl { .. } => {}
            AstNode::ModuleDef { items, .. } => {
                for node in items {
                    self.validate_type_annotations(node)?;
                }
            }
            AstNode::UseDecl {
                path,
                alias: _,
                imports,
                glob,
                span,
                ..
            } => {
                self.validate_use_decl(path, imports, *glob, span)?;
            }
            AstNode::FunctionDef {
                params,
                return_type,
                decorators,
                body,
                span,
                ..
            } => {
                let mut type_vars_in_sig = HashSet::new();
                for p in params {
                    if let Some(ty) = &p.ty {
                        Self::collect_type_vars(ty, &mut type_vars_in_sig);
                        self.validate_type(ty, Some(span))?;
                    }
                }
                if let Some(ret) = return_type {
                    Self::collect_type_vars(ret, &mut type_vars_in_sig);
                    self.validate_type(ret, Some(span))?;
                }
                self.validate_where_bounds(decorators, &type_vars_in_sig, span)?;
                self.validate_type_annotations(body)?;
            }
            AstNode::Closure {
                params,
                return_type,
                body,
                span,
                ..
            } => {
                for p in params {
                    if let Some(ty) = &p.ty {
                        self.validate_type(ty, Some(span))?;
                    }
                }
                if let Some(ret) = return_type {
                    self.validate_type(ret, Some(span))?;
                }
                self.validate_type_annotations(body)?;
            }
            AstNode::StructDef { fields, span, .. } => {
                for (_, ty) in fields {
                    self.validate_type(ty, Some(span))?;
                }
            }
            AstNode::EnumDef { variants, span, .. } => {
                for (_, payload_ty) in variants {
                    if let Some(ty) = payload_ty {
                        self.validate_type(ty, Some(span))?;
                    }
                }
            }
            AstNode::TraitDef {
                name,
                bounds,
                methods,
                span,
                ..
            } => {
                for bound in bounds {
                    if bound == name {
                        return Err(format!(
                            "Trait '{}' cannot bound itself{}",
                            name,
                            Self::format_span(span)
                        ));
                    }
                    if !self.trait_names.contains(bound) {
                        return Err(format!(
                            "Unknown trait bound '{}' on trait '{}'{}",
                            bound,
                            name,
                            Self::format_span(span)
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
            AstNode::LetStmt { ty, expr, span, .. } => {
                if let Some(ty) = ty {
                    self.validate_type(ty, Some(span))?;
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
            AstNode::TryOp { expr, .. } => self.validate_type_annotations(expr)?,
            AstNode::BinaryOp { left, right, .. } => {
                self.validate_type_annotations(left)?;
                self.validate_type_annotations(right)?;
            }
            AstNode::TupleLiteral(items, _) => {
                for item in items {
                    self.validate_type_annotations(item)?;
                }
            }
            AstNode::MemberAccess { object, .. } => self.validate_type_annotations(object)?,
            AstNode::IndexAccess { object, index, .. } => {
                self.validate_type_annotations(object)?;
                self.validate_type_annotations(index)?;
            }
            AstNode::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.validate_type_annotations(expr)?;
                }
            }
            AstNode::ArrayLiteral(items, _) => {
                for item in items {
                    self.validate_type_annotations(item)?;
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

    fn module_key(path: &[String]) -> String {
        path.join("::")
    }

    fn collect_module_index(&mut self, ast: &AstNode, prefix: &[String]) {
        match ast {
            AstNode::Program(nodes) | AstNode::Block(nodes) => {
                for n in nodes {
                    self.collect_module_index(n, prefix);
                }
            }
            AstNode::ModuleDef { name, items, .. } => {
                let mut full = prefix.to_vec();
                full.push(name.clone());
                let key = Self::module_key(&full);
                self.module_decls.insert(key.clone());

                if !prefix.is_empty() {
                    let parent_key = Self::module_key(prefix);
                    self.module_children
                        .entry(parent_key)
                        .or_default()
                        .insert(name.clone());
                }

                for item in items {
                    match item {
                        AstNode::FunctionDef { name, .. }
                        | AstNode::StructDef { name, .. }
                        | AstNode::EnumDef { name, .. }
                        | AstNode::TraitDef { name, .. }
                        | AstNode::ActorDef { name, .. } => {
                            self.module_symbols
                                .entry(key.clone())
                                .or_default()
                                .insert(name.clone());
                        }
                        AstNode::ModuleDef { name, .. } | AstNode::ModuleDecl { name, .. } => {
                            self.module_children
                                .entry(key.clone())
                                .or_default()
                                .insert(name.clone());
                        }
                        _ => {}
                    }
                    self.collect_module_index(item, &full);
                }
            }
            AstNode::ModuleDecl { name, .. } => {
                let mut full = prefix.to_vec();
                full.push(name.clone());
                let key = Self::module_key(&full);
                self.module_decls.insert(key.clone());
                if !prefix.is_empty() {
                    let parent_key = Self::module_key(prefix);
                    self.module_children
                        .entry(parent_key)
                        .or_default()
                        .insert(name.clone());
                }
            }
            _ => {}
        }
    }

    fn bind_imported_symbol(&mut self, symbol: &str, bind_as: &str) {
        if bind_as.is_empty() {
            return;
        }
        if let Some(ty) = self.global_types.get(symbol).cloned() {
            self.global_types.insert(bind_as.to_string(), ty);
        }
    }

    fn collect_use_bindings(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) | AstNode::Block(nodes) => {
                for n in nodes {
                    self.collect_use_bindings(n)?;
                }
            }
            AstNode::ModuleDef { items, .. } => {
                for n in items {
                    self.collect_use_bindings(n)?;
                }
            }
            AstNode::UseDecl {
                path,
                alias,
                imports,
                glob,
                ..
            } => {
                if path.is_empty() {
                    return Ok(());
                }

                let root = &path[0];
                let internal_root_exists = self.module_decls.contains(root);
                if !internal_root_exists {
                    return Ok(());
                }

                if *glob {
                    let module_key = Self::module_key(path);
                    if let Some(symbols) = self.module_symbols.get(&module_key).cloned() {
                        for sym in symbols {
                            self.bind_imported_symbol(&sym, &sym);
                        }
                    }
                    return Ok(());
                }

                if !imports.is_empty() {
                    for (name, maybe_alias) in imports {
                        let bind_as = maybe_alias.as_deref().unwrap_or(name);
                        self.bind_imported_symbol(name, bind_as);
                    }
                    return Ok(());
                }

                if path.len() >= 2 {
                    let leaf = path.last().cloned().unwrap_or_default();
                    let bind_as = alias.as_deref().unwrap_or(&leaf);
                    self.bind_imported_symbol(&leaf, bind_as);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_use_decl(
        &self,
        path: &[String],
        imports: &[(String, Option<String>)],
        glob: bool,
        span: &crate::ast::Span,
    ) -> Result<(), String> {
        if path.is_empty() {
            return Ok(());
        }
        let root = &path[0];
        let internal_root_exists = self.module_decls.contains(root);
        if !internal_root_exists {
            return Ok(());
        }

        let module_key = Self::module_key(path);
        if glob {
            if !self.module_decls.contains(&module_key) {
                return Err(format!(
                    "Unknown module path in use glob: {}{}",
                    module_key,
                    Self::format_span(span)
                ));
            }
            return Ok(());
        }

        if !imports.is_empty() {
            if !self.module_decls.contains(&module_key) {
                return Err(format!(
                    "Unknown module path in grouped use: {}{}",
                    module_key,
                    Self::format_span(span)
                ));
            }
            for (name, _) in imports {
                let symbol_ok = self
                    .module_symbols
                    .get(&module_key)
                    .map(|s| s.contains(name))
                    .unwrap_or(false);
                let child_ok = self
                    .module_children
                    .get(&module_key)
                    .map(|s| s.contains(name))
                    .unwrap_or(false);
                if !symbol_ok && !child_ok {
                    return Err(format!(
                        "Unknown import '{}' in grouped use path {}{}",
                        name,
                        module_key,
                        Self::format_span(span)
                    ));
                }
            }
            return Ok(());
        }

        if path.len() == 1 {
            if !self.module_decls.contains(root) {
                return Err(format!(
                    "Unknown module in use path: {}{}",
                    root,
                    Self::format_span(span)
                ));
            }
            return Ok(());
        }

        let parent = Self::module_key(&path[..path.len() - 1]);
        let leaf = &path[path.len() - 1];
        let symbol_ok = self
            .module_symbols
            .get(&parent)
            .map(|s| s.contains(leaf))
            .unwrap_or(false);
        let child_ok = self
            .module_children
            .get(&parent)
            .map(|s| s.contains(leaf))
            .unwrap_or(false);
        if !symbol_ok && !child_ok {
            return Err(format!(
                "Unknown import path: {}{}",
                Self::module_key(path),
                Self::format_span(span)
            ));
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

    fn collect_type_vars(ty: &Type, out: &mut HashSet<String>) {
        match ty {
            Type::Variable(name) => {
                out.insert(name.clone());
            }
            Type::Generic(_, args) => {
                for arg in args {
                    Self::collect_type_vars(arg, out);
                }
            }
            Type::Function(params, ret) => {
                for p in params {
                    Self::collect_type_vars(p, out);
                }
                Self::collect_type_vars(ret, out);
            }
            Type::Tuple(items) => {
                for item in items {
                    Self::collect_type_vars(item, out);
                }
            }
            Type::Struct(_, fields) => {
                for (_, field_ty) in fields {
                    Self::collect_type_vars(field_ty, out);
                }
            }
            Type::Reference(inner, _) => Self::collect_type_vars(inner, out),
            Type::Primitive(_) | Type::Trait(_) | Type::Actor(_) | Type::Unit => {}
        }
    }

    fn validate_where_bounds(
        &self,
        decorators: &[String],
        signature_type_vars: &HashSet<String>,
        span: &crate::ast::Span,
    ) -> Result<(), String> {
        for d in decorators {
            if let Some(payload) = d.strip_prefix("__where:") {
                let mut parts = payload.splitn(2, ':');
                let type_var = parts.next().unwrap_or_default();
                let traits_raw = parts.next().unwrap_or_default();

                if type_var.is_empty() || traits_raw.is_empty() {
                    return Err(format!(
                        "Invalid where-clause metadata: {}{}",
                        d,
                        Self::format_span(span)
                    ));
                }
                if !signature_type_vars.contains(type_var) {
                    return Err(format!(
                        "Where-clause references unknown type variable '{}' in function signature{}",
                        type_var,
                        Self::format_span(span)
                    ));
                }

                for trait_name in traits_raw.split('+') {
                    if trait_name.is_empty() {
                        return Err(format!(
                            "Invalid where-clause metadata: {}{}",
                            d,
                            Self::format_span(span)
                        ));
                    }
                    if !self.trait_names.contains(trait_name) {
                        return Err(format!(
                            "Unknown trait '{}' in where-clause bound for '{}'{}",
                            trait_name,
                            type_var,
                            Self::format_span(span)
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn bind_type_vars(declared: &Type, actual: &Type, out: &mut HashMap<String, Type>) {
        match declared {
            Type::Variable(v) => {
                out.entry(v.clone()).or_insert_with(|| actual.clone());
            }
            Type::Generic(name, d_args) => {
                if let Type::Generic(a_name, a_args) = actual {
                    if name == a_name && d_args.len() == a_args.len() {
                        for (d, a) in d_args.iter().zip(a_args.iter()) {
                            Self::bind_type_vars(d, a, out);
                        }
                    }
                }
            }
            Type::Tuple(d_items) => {
                if let Type::Tuple(a_items) = actual {
                    for (d, a) in d_items.iter().zip(a_items.iter()) {
                        Self::bind_type_vars(d, a, out);
                    }
                }
            }
            Type::Function(d_params, d_ret) => {
                if let Type::Function(a_params, a_ret) = actual {
                    for (d, a) in d_params.iter().zip(a_params.iter()) {
                        Self::bind_type_vars(d, a, out);
                    }
                    Self::bind_type_vars(d_ret, a_ret, out);
                }
            }
            Type::Reference(d_inner, _) => {
                if let Type::Reference(a_inner, _) = actual {
                    Self::bind_type_vars(d_inner, a_inner, out);
                }
            }
            Type::Primitive(_) | Type::Struct(_, _) | Type::Trait(_) | Type::Actor(_) | Type::Unit => {
            }
        }
    }

    fn concrete_type_name(ty: &Type) -> Option<String> {
        match ty {
            Type::Primitive(n) => Some(n.clone()),
            Type::Struct(n, _) => Some(n.clone()),
            Type::Generic(n, _) => Some(n.clone()),
            Type::Actor(n) => Some(n.clone()),
            _ => None,
        }
    }

    fn satisfies_trait_for_type(&self, trait_name: &str, ty: &Type) -> bool {
        if let Some(type_name) = Self::concrete_type_name(ty) {
            if self
                .trait_impls
                .contains(&(trait_name.to_string(), type_name.clone()))
            {
                return true;
            }
            if self
                .trait_impls
                .iter()
                .any(|(impl_trait, impl_type)| {
                    impl_type == &type_name && self.trait_implies(impl_trait, trait_name)
                })
            {
                return true;
            }
        }

        for gi in &self.generic_trait_impls {
            if !self.trait_implies(&gi.trait_name, trait_name) {
                continue;
            }
            match ty {
                Type::Generic(head, args) if head == &gi.for_type_head => {
                    if gi.for_type_params.len() != args.len() {
                        continue;
                    }
                    let mut mapping: HashMap<String, Type> = HashMap::new();
                    let mut ok = true;
                    for (param, arg) in gi.for_type_params.iter().zip(args.iter()) {
                        if let Some(prev) = mapping.get(param) {
                            if prev != arg {
                                ok = false;
                                break;
                            }
                        } else {
                            mapping.insert(param.clone(), arg.clone());
                        }
                    }
                    if !ok {
                        continue;
                    }

                    for positions in gi.for_type_param_positions.values() {
                        if positions.len() < 2 {
                            continue;
                        }
                        let first_ty = &args[positions[0]];
                        if positions.iter().skip(1).any(|idx| &args[*idx] != first_ty) {
                            ok = false;
                            break;
                        }
                    }
                    if !ok {
                        continue;
                    }
                    for (param, bounds) in &gi.generic_bounds {
                        let arg_ty = match mapping.get(param) {
                            Some(t) => t,
                            None => {
                                ok = false;
                                break;
                            }
                        };
                        for b in bounds {
                            if !self.satisfies_trait_for_type(b, arg_ty) {
                                ok = false;
                                break;
                            }
                        }
                        if !ok {
                            break;
                        }
                    }
                    if ok {
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn trait_implies(&self, derived: &str, required: &str) -> bool {
        if derived == required {
            return true;
        }
        let mut stack = vec![derived.to_string()];
        let mut seen = HashSet::new();
        while let Some(cur) = stack.pop() {
            if !seen.insert(cur.clone()) {
                continue;
            }
            if cur == required {
                return true;
            }
            if let Some(bs) = self.trait_bounds.get(&cur) {
                for b in bs {
                    stack.push(b.clone());
                }
            }
        }
        false
    }

    fn for_type_param_positions(for_type_params: &[String]) -> HashMap<String, Vec<usize>> {
        let mut positions: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, param) in for_type_params.iter().enumerate() {
            positions.entry(param.clone()).or_default().push(idx);
        }
        positions
    }

    fn instantiate_type_with_map(
        ty: &Type,
        map: &mut HashMap<String, Type>,
        checker: &mut TypeChecker,
    ) -> Type {
        match ty {
            Type::Variable(name) => map
                .entry(name.clone())
                .or_insert_with(|| checker.fresh_type_var())
                .clone(),
            Type::Generic(name, args) => Type::Generic(
                name.clone(),
                args.iter()
                    .map(|a| Self::instantiate_type_with_map(a, map, checker))
                    .collect(),
            ),
            Type::Function(params, ret) => Type::Function(
                params
                    .iter()
                    .map(|p| Self::instantiate_type_with_map(p, map, checker))
                    .collect(),
                Box::new(Self::instantiate_type_with_map(ret, map, checker)),
            ),
            Type::Tuple(items) => Type::Tuple(
                items
                    .iter()
                    .map(|t| Self::instantiate_type_with_map(t, map, checker))
                    .collect(),
            ),
            Type::Struct(name, fields) => Type::Struct(
                name.clone(),
                fields
                    .iter()
                    .map(|(f, t)| (f.clone(), Self::instantiate_type_with_map(t, map, checker)))
                    .collect(),
            ),
            Type::Reference(inner, m) => Type::Reference(
                Box::new(Self::instantiate_type_with_map(inner, map, checker)),
                *m,
            ),
            Type::Primitive(_) | Type::Trait(_) | Type::Actor(_) | Type::Unit => ty.clone(),
        }
    }

    fn instantiate_type(&mut self, ty: &Type) -> Type {
        let mut map = HashMap::new();
        Self::instantiate_type_with_map(ty, &mut map, self)
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

    fn with_optional_span(msg: String, span: Option<&crate::ast::Span>) -> String {
        match span {
            Some(s) => format!("{}{}", msg, Self::format_span(s)),
            None => msg,
        }
    }

    fn validate_type(&self, ty: &Type, span: Option<&crate::ast::Span>) -> Result<(), String> {
        match ty {
            Type::Primitive(_) | Type::Variable(_) | Type::Unit | Type::Actor(_) => Ok(()),
            Type::Trait(name) => {
                if self.trait_names.contains(name) {
                    Ok(())
                } else {
                    Err(Self::with_optional_span(
                        format!("Unknown trait type: {}", name),
                        span,
                    ))
                }
            }
            Type::Reference(inner, _) => self.validate_type(inner, span),
            Type::Function(params, ret) => {
                for p in params {
                    self.validate_type(p, span)?;
                }
                self.validate_type(ret, span)
            }
            Type::Tuple(items) => {
                for item in items {
                    self.validate_type(item, span)?;
                }
                Ok(())
            }
            Type::Struct(name, fields) => {
                if !self.global_types.contains_key(name) {
                    return Err(Self::with_optional_span(
                        format!("Unknown struct type: {}", name),
                        span,
                    ));
                }
                for (_, field_ty) in fields {
                    self.validate_type(field_ty, span)?;
                }
                Ok(())
            }
            Type::Generic(name, args) => {
                for arg in args {
                    self.validate_type(arg, span)?;
                }

                if let Some(expected) = Self::builtin_generic_arity(name) {
                    if args.len() != expected {
                        return Err(Self::with_optional_span(
                            format!(
                                "Generic type {} expects {} argument(s), got {}",
                                name,
                                expected,
                                args.len()
                            ),
                            span,
                        ));
                    }
                    return Ok(());
                }

                if let Some(expected) = self.struct_arities.get(name) {
                    if args.len() != *expected {
                        return Err(Self::with_optional_span(
                            format!(
                                "Generic type {} expects {} argument(s), got {}",
                                name,
                                expected,
                                args.len()
                            ),
                            span,
                        ));
                    }
                    return Ok(());
                }

                Err(Self::with_optional_span(
                    format!("Unknown generic type constructor: {}", name),
                    span,
                ))
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
            AstNode::ModuleDecl { .. } => Ok(Type::Unit),
            AstNode::ModuleDef { items, .. } => {
                for node in items {
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
            AstNode::TupleLiteral(items, _) => {
                let mut tys = Vec::with_capacity(items.len());
                for item in items {
                    tys.push(self.collect(item, env)?);
                }
                Ok(Type::Tuple(tys))
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
            AstNode::CharLiteral(_, _) => Ok(Type::Primitive("char".to_string())),
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
            AstNode::FunctionCall {
                func, args, span, ..
            } => {
                let mut f_ty = self.collect(func, env)?;
                let res_ty = self.fresh_type_var();
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.collect(arg, env)?);
                }

                if let AstNode::Identifier(func_name, _) = &**func {
                    if let Some(global_func_ty) = self.global_types.get(func_name).cloned() {
                        if matches!(global_func_ty, Type::Function(_, _)) {
                            f_ty = self.instantiate_type(&global_func_ty);
                        }
                    }

                    if let Some(bounds) = self.function_where_bounds.get(func_name) {
                        if let Some(Type::Function(param_tys, _)) =
                            self.global_types.get(func_name).cloned()
                        {
                            let mut mapping = HashMap::new();
                            for (decl, actual) in param_tys.iter().zip(arg_types.iter()) {
                                Self::bind_type_vars(decl, actual, &mut mapping);
                            }
                            for (type_var, traits) in bounds {
                                let bound_ty = match mapping.get(type_var) {
                                    Some(t) => t,
                                    None => {
                                        return Err(format!(
                                            "Cannot resolve where-bound type variable '{}' at call site{}",
                                            type_var,
                                            Self::format_span(span)
                                        ))
                                    }
                                };
                                for trait_name in traits {
                                    if !self.satisfies_trait_for_type(trait_name, bound_ty) {
                                        return Err(format!(
                                            "Type '{}' does not implement required trait '{}' for where-bound '{}'{}",
                                            bound_ty,
                                            trait_name,
                                            type_var,
                                            Self::format_span(span)
                                        ));
                                    }
                                }
                            }
                        }
                    }
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
            AstNode::TryOp { expr, span } => {
                let inner_ty = self.collect(expr, env)?;
                match inner_ty {
                    Type::Generic(name, args) if name == "Option" && args.len() == 1 => {
                        Ok(args[0].clone())
                    }
                    Type::Generic(name, args) if name == "Result" && args.len() == 2 => {
                        Ok(args[0].clone())
                    }
                    _ => Err(format!(
                        "Try operator requires Option<T> or Result<T, E>, got {:?} at line {} col {}",
                        inner_ty, span.line, span.col
                    )),
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
                scrutinee, arms, span, ..
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
                self.ensure_match_exhaustive(&s_ty, arms, span)?;
                Ok(res_ty)
            }
            AstNode::Closure {
                params,
                return_type,
                body,
                ..
            } => {
                let mut local_env = env.clone().extend();
                let mut param_types = Vec::new();
                for p in params {
                    let ty = p.ty.clone().unwrap_or_else(|| self.fresh_type_var());
                    local_env.bind(p.name.clone(), ty.clone());
                    param_types.push(ty);
                }

                let body_ty = self.collect(body, &mut local_env)?;
                let ret_ty = if let Some(declared_ret) = return_type {
                    self.constraints.push(Constraint {
                        left: body_ty.clone(),
                        right: declared_ret.clone(),
                    });
                    declared_ret.clone()
                } else {
                    body_ty
                };

                Ok(Type::Function(param_types, Box::new(ret_ty)))
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
            AstNode::ArrayLiteral(items, _) => {
                if items.is_empty() {
                    return Ok(Type::Generic("Vec".to_string(), vec![self.fresh_type_var()]));
                }
                let first_ty = self.collect(&items[0], env)?;
                for item in &items[1..] {
                    let ty = self.collect(item, env)?;
                    self.constraints.push(Constraint {
                        left: first_ty.clone(),
                        right: ty,
                    });
                }
                Ok(Type::Generic("Vec".to_string(), vec![first_ty]))
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
            AstNode::IndexAccess {
                object, index, span, ..
            } => {
                let obj_ty = self.collect(object, env)?;
                let idx_ty = self.collect(index, env)?;
                self.constraints.push(Constraint {
                    left: idx_ty,
                    right: Type::Primitive("i32".to_string()),
                });
                match obj_ty {
                    Type::Generic(name, args) if name == "Vec" && args.len() == 1 => {
                        Ok(args[0].clone())
                    }
                    Type::Variable(_) => Ok(self.fresh_type_var()),
                    _ => Err(format!(
                        "Cannot index into non-indexable type {:?} at line {} col {}",
                        obj_ty, span.line, span.col
                    )),
                }
            }
            AstNode::ActorDef { name, .. } => {
                let ty = Type::Actor(name.clone());
                self.global_types.insert(name.clone(), ty.clone());
                if let Some(msg_ty) = self.infer_actor_message_type(ast, env)? {
                    self.actor_message_types.insert(name.clone(), msg_ty);
                }
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
                let m_ty = self.collect(message, env)?;

                match t_ty {
                    Type::Actor(actor_name) => {
                        if let Some(expected_msg_ty) = self.actor_message_types.get(&actor_name).cloned() {
                            self.constraints.push(Constraint {
                                left: m_ty,
                                right: expected_msg_ty,
                            });
                        }
                        Ok(Type::Unit)
                    }
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
            AstNode::UseDecl { .. } => Ok(Type::Unit),
            AstNode::MacroCall { name, args, span } => {
                if !self.macro_names.contains(name) {
                    return Err(format!(
                        "Unknown macro '{}' at line {} col {}",
                        name, span.line, span.col
                    ));
                }
                for arg in args {
                    self.collect(arg, env)?;
                }
                Ok(Type::Unit)
            }
            _ => Ok(Type::Unit),
        }
    }

    fn infer_actor_message_type(
        &mut self,
        actor_node: &AstNode,
        env: &mut TypeEnv,
    ) -> Result<Option<Type>, String> {
        let body = match actor_node {
            AstNode::ActorDef { body, .. } => body,
            _ => return Ok(None),
        };
        let stmts = match &**body {
            AstNode::Block(stmts) => stmts,
            _ => return Ok(None),
        };

        let mut found: Option<Type> = None;
        for stmt in stmts {
            let arms = match stmt {
                AstNode::Receive { arms, .. } => arms,
                _ => continue,
            };
            for arm in arms {
                let mut penv = env.clone().extend();
                let p_ty = self.collect_pattern(&arm.pattern, &mut penv)?;
                if let Some(prev) = &found {
                    self.constraints.push(Constraint {
                        left: prev.clone(),
                        right: p_ty.clone(),
                    });
                } else {
                    found = Some(p_ty.clone());
                }
            }
        }

        Ok(found)
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
                let mut tys = Vec::new();
                for item in items {
                    tys.push(self.collect_pattern(item, env)?);
                }
                Ok(Type::Tuple(tys))
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
        span: &crate::ast::Span,
    ) -> Result<(), String> {
        if scrutinee_ty == &Type::Primitive("bool".to_string()) {
            let mut has_true = false;
            let mut has_false = false;
            let mut has_wildcard = false;

            for arm in arms {
                if arm.guard.is_some() {
                    continue;
                }
                let (t, f, w) = Self::bool_pattern_coverage(&arm.pattern);
                has_true |= t;
                has_false |= f;
                has_wildcard |= w;
            }

            if has_wildcard || (has_true && has_false) {
                return Ok(());
            }
            return Err(format!(
                "Non-exhaustive match for bool at line {} col {}: expected true and false branches (or _)",
                span.line, span.col
            ));
        }

        if let Type::Tuple(items) = scrutinee_ty {
            let domains: Option<Vec<Vec<String>>> = items
                .iter()
                .map(|t| self.finite_domain_for_type_ctx(t))
                .collect();

            if let Some(domains) = domains {
                if !domains.is_empty() {
                    let universe: usize = domains.iter().map(|d| d.len()).product();
                    let mut covered = BTreeSet::new();
                    for arm in arms {
                        if arm.guard.is_some() {
                            continue;
                        }
                        for key in Self::tuple_finite_pattern_coverage(&arm.pattern, &domains) {
                            covered.insert(key);
                        }
                    }
                    if covered.len() == universe {
                        return Ok(());
                    }
                    return Err(format!(
                        "Non-exhaustive match for finite tuple at line {} col {}",
                        span.line, span.col
                    ));
                }
            }
        }

        let struct_type = match scrutinee_ty {
            Type::Struct(name, fields) => Some((name.clone(), fields.clone())),
            Type::Variable(name) => self.global_types.get(name).and_then(|t| {
                if let Type::Struct(s_name, s_fields) = t {
                    Some((s_name.clone(), s_fields.clone()))
                } else {
                    None
                }
            }),
            _ => None,
        };

        if let Some((name, fields)) = struct_type {
            if let Some(domain) = self.finite_domain_for_type_ctx(&Type::Struct(name.clone(), fields.clone())) {
                let mut covered = BTreeSet::new();
                for arm in arms {
                    if arm.guard.is_some() {
                        continue;
                    }
                    for val in Self::domain_values_for_pattern(&arm.pattern, &domain) {
                        covered.insert(val);
                    }
                }
                if covered.len() == domain.len() {
                    return Ok(());
                }
                return Err(format!(
                    "Non-exhaustive match for struct {} at line {} col {}",
                    name, span.line, span.col
                ));
            }
        }

        if let Type::Primitive(enum_name) = scrutinee_ty {
            if self.enum_variants.contains_key(enum_name) {
                return self.ensure_enum_exhaustive_for_name(enum_name, arms, span);
            }
        }

        if let Type::Variable(enum_name) = scrutinee_ty {
            if self.enum_variants.contains_key(enum_name) {
                return self.ensure_enum_exhaustive_for_name(enum_name, arms, span);
            }
        }

        Ok(())
    }

    fn ensure_enum_exhaustive_for_name(
        &self,
        enum_name: &str,
        arms: &[MatchArm],
        span: &crate::ast::Span,
    ) -> Result<(), String> {
        let variants = match self.enum_variants.get(enum_name) {
            Some(v) => v,
            None => return Ok(()),
        };

        let mut covered: HashMap<String, bool> =
            variants.iter().map(|v| (v.clone(), false)).collect();
        let mut wildcard = false;
        let mut payload_covered: HashMap<String, BTreeSet<String>> = HashMap::new();
        let mut non_finite_payload_catchall: HashMap<String, bool> = variants
            .iter()
            .map(|v| (v.clone(), false))
            .collect();

        for arm in arms {
            if arm.guard.is_some() {
                continue;
            }
            for variant in Self::enum_covered_variants(enum_name, &arm.pattern) {
                covered.insert(variant, true);
            }
            if Self::pattern_has_wildcard(&arm.pattern) {
                wildcard = true;
            }

            for variant in variants {
                match self.enum_variant_payload_domain(enum_name, variant) {
                    Some(Some(domain)) => {
                        let vals = Self::enum_payload_values_for_pattern(
                            enum_name,
                            variant,
                            &arm.pattern,
                            &domain,
                        );
                        if !vals.is_empty() {
                            payload_covered
                                .entry(variant.clone())
                                .or_default()
                                .extend(vals);
                        }
                    }
                    Some(None) => {
                        if Self::enum_payload_has_catchall_for_variant(
                            enum_name,
                            variant,
                            &arm.pattern,
                        ) {
                            non_finite_payload_catchall.insert(variant.clone(), true);
                        }
                    }
                    None => {}
                }
            }
        }

        if !wildcard && !covered.values().all(|v| *v) {
            let missing: Vec<String> = covered
                .iter()
                .filter_map(|(v, is_covered)| if !is_covered { Some(v.clone()) } else { None })
                .collect();
            return Err(format!(
                "Non-exhaustive match for enum {} at line {} col {}: missing variant(s): {}",
                enum_name,
                span.line,
                span.col,
                missing.join(", ")
            ));
        }

        if !wildcard {
            for variant in variants {
                match self.enum_variant_payload_domain(enum_name, variant) {
                    Some(Some(domain)) => {
                        let covered_vals = payload_covered.get(variant).cloned().unwrap_or_default();
                        if covered_vals.len() != domain.len() {
                            return Err(format!(
                                "Non-exhaustive payload match for enum {}::{} at line {} col {}",
                                enum_name, variant, span.line, span.col
                            ));
                        }
                    }
                    Some(None) => {
                        if !non_finite_payload_catchall
                            .get(variant)
                            .copied()
                            .unwrap_or(false)
                        {
                            return Err(format!(
                                "Non-exhaustive payload match for enum {}::{} at line {} col {}: non-finite payload requires catch-all payload pattern",
                                enum_name, variant, span.line, span.col
                            ));
                        }
                    }
                    None => {}
                }
            }
        }

        Ok(())
    }

    fn enum_payload_has_catchall_for_variant(
        enum_name: &str,
        variant: &str,
        pattern: &Pattern,
    ) -> bool {
        match pattern {
            Pattern::Enum(p_enum, p_variant, payload) if p_enum == enum_name && p_variant == variant => {
                match payload {
                    None => true,
                    Some(p) => Self::pattern_is_catchall_payload(p),
                }
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .any(|p| Self::enum_payload_has_catchall_for_variant(enum_name, variant, p)),
            Pattern::Underscore | Pattern::Identifier(_) => true,
            _ => false,
        }
    }

    fn pattern_is_catchall_payload(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Underscore | Pattern::Identifier(_) => true,
            Pattern::Or(patterns) => patterns.iter().any(Self::pattern_is_catchall_payload),
            _ => false,
        }
    }

    fn finite_domain_for_type_ctx(&self, ty: &Type) -> Option<Vec<String>> {
        self.finite_domain_for_type_ctx_seen(ty, &mut HashSet::new())
    }

    fn finite_domain_for_type_ctx_seen(
        &self,
        ty: &Type,
        visiting: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        if ty == &Type::Primitive("bool".to_string()) {
            return Some(vec!["false".to_string(), "true".to_string()]);
        }
        if let Type::Variable(name) = ty {
            if name == "bool" {
                return Some(vec!["false".to_string(), "true".to_string()]);
            }
            if self.enum_variants.contains_key(name) {
                return self.finite_domain_for_enum(name, visiting);
            }
            if let Some(mapped) = self.global_types.get(name) {
                if mapped != ty {
                    if !visiting.insert(name.clone()) {
                        return Some(vec![Self::RECURSIVE_DOMAIN_WILDCARD.to_string()]);
                    }
                    let out = self.finite_domain_for_type_ctx_seen(mapped, visiting);
                    visiting.remove(name);
                    return out;
                }
            }
        }
        if let Type::Primitive(enum_name) = ty {
            if self.enum_variants.contains_key(enum_name) {
                return self.finite_domain_for_enum(enum_name, visiting);
            }
        }
        if let Type::Tuple(items) = ty {
            let mut domains = Vec::new();
            for item in items {
                domains.push(self.finite_domain_for_type_ctx_seen(item, visiting)?);
            }
            let mut out = Vec::new();
            Self::cross_product_domains(&domains, 0, &mut Vec::new(), &mut out);
            return Some(
                out.into_iter()
                    .map(|vals| {
                        let encoded: Vec<String> = vals
                            .iter()
                            .map(|v| Self::encode_domain_atom(v))
                            .collect();
                        format!("t:{}", encoded.join("\u{1f}"))
                    })
                    .collect(),
            );
        }
        if let Type::Struct(name, fields) = ty {
            if fields.is_empty() {
                return Some(vec![format!("s:{}:", name)]);
            }
            let mut domains = Vec::new();
            let mut field_names = Vec::new();
            for (field, field_ty) in fields {
                field_names.push(field.clone());
                domains.push(self.finite_domain_for_type_ctx_seen(field_ty, visiting)?);
            }
            let mut out = Vec::new();
            Self::cross_product_domains(&domains, 0, &mut Vec::new(), &mut out);
            return Some(
                out.into_iter()
                    .map(|vals| {
                        let pairs: Vec<String> = field_names
                            .iter()
                            .cloned()
                            .zip(vals)
                            .map(|(k, v)| format!("{}={}", k, Self::encode_domain_atom(&v)))
                            .collect();
                        format!("s:{}:{}", name, pairs.join(";"))
                    })
                    .collect(),
            );
        }
        None
    }

    fn finite_domain_for_enum(
        &self,
        enum_name: &str,
        visiting: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        let variants = self.enum_variants.get(enum_name)?;
        if !visiting.insert(enum_name.to_string()) {
            return Some(
                variants
                    .iter()
                    .map(|variant| {
                        let has_payload = self
                            .enum_variant_payloads
                            .get(enum_name)
                            .and_then(|m| m.get(variant))
                            .cloned()
                            .flatten()
                            .is_some();
                        if has_payload {
                            format!(
                                "{}::{}({})",
                                enum_name,
                                variant,
                                Self::RECURSIVE_DOMAIN_WILDCARD
                            )
                        } else {
                            format!("{}::{}", enum_name, variant)
                        }
                    })
                    .collect(),
            );
        }
        let mut out = Vec::new();
        for variant in variants {
            let payload_ty = self
                .enum_variant_payloads
                .get(enum_name)
                .and_then(|m| m.get(variant))
                .cloned()
                .flatten();
            if let Some(payload_ty) = payload_ty {
                let payload_domain = self.finite_domain_for_type_ctx_seen(&payload_ty, visiting)?;
                for payload_value in payload_domain {
                    out.push(format!("{}::{}({})", enum_name, variant, payload_value));
                }
            } else {
                out.push(format!("{}::{}", enum_name, variant));
            }
        }
        visiting.remove(enum_name);
        Some(out)
    }

    fn cross_product_domains(
        domains: &[Vec<String>],
        idx: usize,
        cur: &mut Vec<String>,
        out: &mut Vec<Vec<String>>,
    ) {
        if idx == domains.len() {
            out.push(cur.clone());
            return;
        }
        for v in &domains[idx] {
            cur.push(v.clone());
            Self::cross_product_domains(domains, idx + 1, cur, out);
            cur.pop();
        }
    }

    fn encode_domain_atom(value: &str) -> String {
        let mut out = String::new();
        for ch in value.chars() {
            match ch {
                '%' => out.push_str("%25"),
                '\u{1f}' => out.push_str("%1F"),
                ';' => out.push_str("%3B"),
                '=' => out.push_str("%3D"),
                '(' => out.push_str("%28"),
                ')' => out.push_str("%29"),
                _ => out.push(ch),
            }
        }
        out
    }

    fn decode_domain_atom(value: &str) -> String {
        let mut out = String::new();
        let bytes = value.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let h1 = bytes[i + 1] as char;
                let h2 = bytes[i + 2] as char;
                let hex = [h1, h2].iter().collect::<String>();
                if let Ok(v) = u8::from_str_radix(&hex, 16) {
                    out.push(v as char);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    fn enum_variant_payload_domain(
        &self,
        enum_name: &str,
        variant: &str,
    ) -> Option<Option<Vec<String>>> {
        let payload_ty = self
            .enum_variant_payloads
            .get(enum_name)
            .and_then(|m| m.get(variant))
            .cloned();
        match payload_ty {
            None => None,
            Some(None) => Some(None),
            Some(Some(ty)) => Some(self.finite_domain_for_type_ctx(&ty)),
        }
    }

    fn enum_payload_values_for_pattern(
        enum_name: &str,
        variant: &str,
        pattern: &Pattern,
        domain: &[String],
    ) -> Vec<String> {
        match pattern {
            Pattern::Enum(p_enum, p_variant, payload) if p_enum == enum_name && p_variant == variant => {
                match payload {
                    None => domain.to_vec(),
                    Some(p) => Self::domain_values_for_pattern(p, domain),
                }
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .flat_map(|p| Self::enum_payload_values_for_pattern(enum_name, variant, p, domain))
                .collect(),
            Pattern::Underscore | Pattern::Identifier(_) => domain.to_vec(),
            _ => Vec::new(),
        }
    }

    fn domain_values_for_pattern(pattern: &Pattern, domain: &[String]) -> Vec<String> {
        match pattern {
            Pattern::Underscore | Pattern::Identifier(_) => domain.to_vec(),
            Pattern::BoolLiteral(true) => domain
                .iter()
                .filter(|v| v.as_str() == "true")
                .cloned()
                .collect(),
            Pattern::BoolLiteral(false) => domain
                .iter()
                .filter(|v| v.as_str() == "false")
                .cloned()
                .collect(),
            Pattern::Enum(enum_name, variant, payload) => {
                let mut out = Vec::new();
                let head = format!("{}::{}", enum_name, variant);
                for entry in domain {
                    if entry == &head {
                        if payload.is_none() {
                            out.push(entry.clone());
                        }
                        continue;
                    }
                    if let Some(rest) = entry.strip_prefix(&head) {
                        let payload_pat = match payload {
                            Some(p) => p.as_ref(),
                            None => continue,
                        };
                        if !(rest.starts_with('(') && rest.ends_with(')')) {
                            continue;
                        }
                        let inner = &rest[1..rest.len() - 1];
                        let selected = Self::domain_values_for_pattern(
                            payload_pat,
                            &[inner.to_string()],
                        );
                        if !selected.is_empty() {
                            out.push(entry.clone());
                        }
                    }
                }
                out
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .flat_map(|p| Self::domain_values_for_pattern(p, domain))
                .collect(),
            Pattern::Tuple(items) => {
                let tuple_entries: Vec<Vec<String>> = domain
                    .iter()
                    .filter_map(|entry| {
                        entry.strip_prefix("t:").map(|rest| {
                            rest.split('\u{1f}')
                                .map(Self::decode_domain_atom)
                                .collect()
                        })
                    })
                    .collect();
                if tuple_entries.is_empty() {
                    return Vec::new();
                }
                let arity = tuple_entries[0].len();
                if items.len() != arity {
                    return Vec::new();
                }

                let mut slot_domains: Vec<Vec<String>> = vec![Vec::new(); arity];
                for entry in &tuple_entries {
                    if entry.len() != arity {
                        return Vec::new();
                    }
                    for (i, v) in entry.iter().enumerate() {
                        if !slot_domains[i].contains(v) {
                            slot_domains[i].push(v.clone());
                        }
                    }
                }

                let mut slot_values: Vec<Vec<String>> = Vec::new();
                for (pat, slot_domain) in items.iter().zip(slot_domains.iter()) {
                    let vals = Self::domain_values_for_pattern(pat, slot_domain);
                    if vals.is_empty() {
                        return Vec::new();
                    }
                    slot_values.push(vals);
                }

                fn combine(
                    slots: &[Vec<String>],
                    idx: usize,
                    cur: &mut Vec<String>,
                    out: &mut Vec<String>,
                ) {
                    if idx == slots.len() {
                        let encoded: Vec<String> = cur
                            .iter()
                            .map(|v| TypeChecker::encode_domain_atom(v))
                            .collect();
                        out.push(format!("t:{}", encoded.join("\u{1f}")));
                        return;
                    }
                    for v in &slots[idx] {
                        cur.push(v.clone());
                        combine(slots, idx + 1, cur, out);
                        cur.pop();
                    }
                }

                let mut combined = Vec::new();
                let mut cur = Vec::new();
                combine(&slot_values, 0, &mut cur, &mut combined);
                let set: HashSet<String> = combined.into_iter().collect();
                domain
                    .iter()
                    .filter(|v| set.contains(*v))
                    .cloned()
                    .collect()
            }
            Pattern::Struct(struct_name, fields) => {
                let struct_entries: Vec<(String, HashMap<String, String>)> = domain
                    .iter()
                    .filter_map(|entry| {
                        let body = entry.strip_prefix(&format!("s:{}:", struct_name))?;
                        let mut m = HashMap::new();
                        if body.is_empty() {
                            return Some((entry.clone(), m));
                        }
                        for pair in body.split(';') {
                            let mut it = pair.splitn(2, '=');
                            let k = it.next()?.to_string();
                            let v = Self::decode_domain_atom(it.next()?);
                            m.insert(k, v);
                        }
                        Some((entry.clone(), m))
                    })
                    .collect();
                if struct_entries.is_empty() {
                    return Vec::new();
                }

                let mut field_domains: HashMap<String, Vec<String>> = HashMap::new();
                for (_, entry) in &struct_entries {
                    for (k, v) in entry {
                        field_domains
                            .entry(k.clone())
                            .or_default()
                            .push(v.clone());
                    }
                }
                for vals in field_domains.values_mut() {
                    vals.sort();
                    vals.dedup();
                }

                let mut selectors: Vec<(String, Vec<String>)> = Vec::new();
                for (field_name, pat) in fields {
                    let d = field_domains.get(field_name).cloned().unwrap_or_default();
                    let vals = Self::domain_values_for_pattern(pat, &d);
                    if vals.is_empty() {
                        return Vec::new();
                    }
                    selectors.push((field_name.clone(), vals));
                }

                let mut allowed = HashSet::new();
                for (entry_raw, entry) in &struct_entries {
                    let mut ok = true;
                    for (k, vals) in &selectors {
                        match entry.get(k) {
                            Some(v) if vals.contains(v) => {}
                            _ => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        allowed.insert(entry_raw.clone());
                    }
                }

                domain
                    .iter()
                    .filter(|v| allowed.contains(*v))
                    .cloned()
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn tuple_finite_pattern_coverage(pattern: &Pattern, domains: &[Vec<String>]) -> Vec<String> {
        match pattern {
            Pattern::Underscore | Pattern::Identifier(_) => {
                fn combine(domains: &[Vec<String>], idx: usize, cur: &mut Vec<String>, out: &mut Vec<String>) {
                    if idx == domains.len() {
                        out.push(cur.join("\u{1f}"));
                        return;
                    }
                    for v in &domains[idx] {
                        cur.push(v.clone());
                        combine(domains, idx + 1, cur, out);
                        cur.pop();
                    }
                }
                let mut out = Vec::new();
                let mut cur = Vec::new();
                combine(domains, 0, &mut cur, &mut out);
                out
            }
            Pattern::Or(patterns) => patterns
                .iter()
                .flat_map(|p| Self::tuple_finite_pattern_coverage(p, domains))
                .collect(),
            Pattern::Tuple(items) if items.len() == domains.len() => {
                let mut slot_values: Vec<Vec<String>> = Vec::new();
                for (item, domain) in items.iter().zip(domains.iter()) {
                    let vals = Self::domain_values_for_pattern(item, domain);
                    if vals.is_empty() {
                        return Vec::new();
                    }
                    slot_values.push(vals);
                }
                fn combine(
                    slots: &[Vec<String>],
                    idx: usize,
                    cur: &mut Vec<String>,
                    out: &mut Vec<String>,
                ) {
                    if idx == slots.len() {
                        out.push(cur.join("\u{1f}"));
                        return;
                    }
                    for v in &slots[idx] {
                        cur.push(v.clone());
                        combine(slots, idx + 1, cur, out);
                        cur.pop();
                    }
                }
                let mut out = Vec::new();
                let mut cur = Vec::new();
                combine(&slot_values, 0, &mut cur, &mut out);
                out
            }
            _ => Vec::new(),
        }
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
                (Type::Tuple(t1), Type::Tuple(t2)) => {
                    if t1.len() != t2.len() {
                        return Err("Tuple length mismatch".to_string());
                    }
                    for (a, b) in t1.into_iter().zip(t2.into_iter()) {
                        constraints.push(Constraint { left: a, right: b });
                    }
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
            Type::Tuple(items) => {
                Type::Tuple(items.iter().map(|t| self.apply_subst(t, substitution)).collect())
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
            Type::Tuple(items) => items.iter().any(|t| self.occurs_check(var, t)),
            _ => false,
        }
    }
}
