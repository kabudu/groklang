use crate::ast::AstNode;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Value as IrValue};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::Context as CodegenContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

pub struct Compiler {
    builder_context: FunctionBuilderContext,
    ctx: CodegenContext,
    module: JITModule,
}

impl Compiler {
    pub fn new() -> Self {
        let builder_context = FunctionBuilderContext::new();

        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();

        // On aarch64, we need to be careful with PLT.
        // Using the native builder is better.
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(jit_builder);

        Self {
            builder_context,
            ctx: CodegenContext::new(),
            module,
        }
    }

    pub fn compile_program(&mut self, ast: &AstNode) -> Result<*const u8, String> {
        if let AstNode::Program(nodes) = ast {
            // First pass: Declare all functions
            for node in nodes {
                if let AstNode::FunctionDef { name, params, .. } = node {
                    let mut sig = self.module.make_signature();
                    sig.returns.push(AbiParam::new(types::I64));
                    for _ in params {
                        sig.params.push(AbiParam::new(types::I64));
                    }
                    self.module
                        .declare_function(name, Linkage::Export, &sig)
                        .map_err(|e| e.to_string())?;
                }
            }

            // Second pass: Define all functions
            for node in nodes {
                if let AstNode::FunctionDef {
                    name, params, body, ..
                } = node
                {
                    self.compile_function(name, params, body)?;
                }
            }

            // Finalize all functions
            let _ = self.module.finalize_definitions();

            // Return address of 'main' if it exists
            let main_id = self
                .module
                .get_name("main")
                .ok_or("No 'main' function found")?;
            if let cranelift_module::FuncOrDataId::Func(id) = main_id {
                Ok(self.module.get_finalized_function(id))
            } else {
                Err("main is not a function".to_string())
            }
        } else {
            Err("Expected Program node".to_string())
        }
    }

    fn compile_function(
        &mut self,
        name: &str,
        params: &Vec<crate::ast::Param>,
        body: &AstNode,
    ) -> Result<(), String> {
        let func_id =
            if let Some(cranelift_module::FuncOrDataId::Func(id)) = self.module.get_name(name) {
                id
            } else {
                return Err(format!("Function {} not declared", name));
            };

        // Reset context and set signature
        self.module.clear_context(&mut self.ctx);
        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I64));
        for _ in params {
            self.ctx
                .func
                .signature
                .params
                .push(AbiParam::new(types::I64));
        }

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut variables = HashMap::new();
        for (i, param) in params.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
            let variable = Variable::from_u32(i as u32);
            builder.declare_var(variable, types::I64);
            builder.def_var(variable, val);
            variables.insert(param.name.clone(), variable);
        }

        let mut translator = Translator {
            builder,
            variables,
            module: &mut self.module,
        };

        let return_val = translator.translate_node(body)?;
        translator.builder.ins().return_(&[return_val]);
        translator.builder.finalize();

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

struct Translator<'a> {
    builder: FunctionBuilder<'a>,
    variables: HashMap<String, Variable>,
    module: &'a mut JITModule,
}

impl<'a> Translator<'a> {
    fn translate_node(&mut self, node: &AstNode) -> Result<IrValue, String> {
        match node {
            AstNode::IntLiteral(val, _) => Ok(self.builder.ins().iconst(types::I64, *val)),
            AstNode::BinaryOp {
                left, op, right, ..
            } => {
                let lhs = self.translate_node(left)?;
                let rhs = self.translate_node(right)?;
                match op.as_str() {
                    "+" => Ok(self.builder.ins().iadd(lhs, rhs)),
                    "-" => Ok(self.builder.ins().isub(lhs, rhs)),
                    "*" => Ok(self.builder.ins().imul(lhs, rhs)),
                    "==" => {
                        let cmp = self.builder.ins().icmp(IntCC::Equal, lhs, rhs);
                        Ok(self.builder.ins().uextend(types::I64, cmp))
                    }
                    _ => Err(format!("Unsupported operator: {}", op)),
                }
            }
            AstNode::Identifier(name, _) => {
                let var = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Unknown variable: {}", name))?;
                Ok(self.builder.use_var(*var))
            }
            AstNode::FunctionCall { func, args, .. } => {
                if let AstNode::Identifier(name, _) = &**func {
                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(self.translate_node(arg)?);
                    }

                    let func_id = self
                        .module
                        .get_name(name)
                        .ok_or_else(|| format!("Function {} not found", name))?;

                    if let cranelift_module::FuncOrDataId::Func(id) = func_id {
                        let local_func =
                            self.module.declare_func_in_func(id, &mut self.builder.func);
                        let call = self.builder.ins().call(local_func, &arg_vals);
                        Ok(self.builder.inst_results(call)[0])
                    } else {
                        Err(format!("{} is not a function", name))
                    }
                } else {
                    Err("Direct function calls only supported via identifier".to_string())
                }
            }
            AstNode::LetStmt { name, expr, .. } => {
                let val = self.translate_node(expr)?;
                let variable = Variable::from_u32(self.variables.len() as u32);
                self.builder.declare_var(variable, types::I64);
                self.builder.def_var(variable, val);
                self.variables.insert(name.clone(), variable);
                Ok(val)
            }
            AstNode::IfExpr {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond_val = self.translate_node(condition)?;

                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                // Add parameter to merge block for the result
                self.builder.append_block_param(merge_block, types::I64);

                self.builder
                    .ins()
                    .brif(cond_val, then_block, &[], else_block, &[]);

                // Then
                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                let then_result = self.translate_node(then_body)?;
                self.builder.ins().jump(merge_block, &[then_result]);

                // Else
                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                let else_result = if let Some(eb) = else_body {
                    self.translate_node(eb)?
                } else {
                    self.builder.ins().iconst(types::I64, 0)
                };
                self.builder.ins().jump(merge_block, &[else_result]);

                // Merge
                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
                Ok(self.builder.block_params(merge_block)[0])
            }
            AstNode::Block(stmts) => {
                let mut last_val = self.builder.ins().iconst(types::I64, 0);
                for stmt in stmts {
                    last_val = self.translate_node(stmt)?;
                }
                Ok(last_val)
            }
            _ => Err(format!(
                "Cranelift translation not implemented for node: {:?}",
                node
            )),
        }
    }
}
