use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature, UserFuncName, condcodes::IntCC};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use cranelift_native;

use crate::optimizations::{SpecializedFunction, SpecializedOpcode};
use crate::ir::Opcode;

/// The JIT compiler for GrokLang, powered by Cranelift.
pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    ctx: cranelift_codegen::Context,
    module: JITModule,
}

impl JITCompiler {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();
        
        let isa_builder = cranelift_native::builder().expect("host machine is not supported");
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .expect("failed to finish isa builder");

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }

    /// Compiles a SpecializedFunction into native machine code.
    pub fn compile(&mut self, func_def: &SpecializedFunction) -> Result<*const u8, String> {
        self.module.clear_context(&mut self.ctx);

        let mut sig = Signature::new(CallConv::SystemV);
        for _ in 0..func_def.params.len() {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));

        self.ctx.func.signature = sig;
        self.ctx.func.name = UserFuncName::default();

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        
        let mut cl_blocks = std::collections::HashMap::new();
        for func_block in &func_def.blocks {
            let block = builder.create_block();
            cl_blocks.insert(func_block.label.clone(), block);
        }

        let mut vars = std::collections::HashMap::new();
        for i in 0..128 {
            let var = Variable::from_u32(i as u32);
            builder.declare_var(var, types::I64);
            vars.insert(i, var);
        }

        let entry_block = *cl_blocks.get("entry").ok_or("No entry block found")?;
        builder.append_block_params_for_function_params(entry_block);
        
        for (i, func_block) in func_def.blocks.iter().enumerate() {
            let cl_block = cl_blocks[&func_block.label];
            builder.switch_to_block(cl_block);
            
            if func_block.label == "entry" {
                for (idx, _param) in func_def.params.iter().enumerate() {
                    let val = builder.block_params(entry_block)[idx];
                    builder.def_var(vars[&idx], val);
                }
            }

            let mut stack = Vec::new();
            let mut terminated = false;
            
            for instr in &func_block.instructions {
                if terminated { break; }
                
                match &instr.opcode {
                    SpecializedOpcode::PushSmallInt(v) => {
                        stack.push(builder.ins().iconst(types::I64, *v));
                    }
                    SpecializedOpcode::Generic(Opcode::PushInt(v)) => {
                        stack.push(builder.ins().iconst(types::I64, *v));
                    }
                    SpecializedOpcode::LoadLocalFast(idx) => {
                        stack.push(builder.use_var(vars[idx]));
                    }
                    SpecializedOpcode::StoreLocalFast(idx) => {
                        let val = stack.pop().ok_or("Stack underflow")?;
                        builder.def_var(vars[idx], val);
                    }
                    SpecializedOpcode::IntAdd => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        stack.push(builder.ins().iadd(a, b));
                    }
                    SpecializedOpcode::IntSub => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        stack.push(builder.ins().isub(a, b));
                    }
                    SpecializedOpcode::IntMul => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        stack.push(builder.ins().imul(a, b));
                    }
                    SpecializedOpcode::IntDiv => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        stack.push(builder.ins().sdiv(a, b));
                    }
                    SpecializedOpcode::IntLt => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        let cmp = builder.ins().icmp(IntCC::SignedLessThan, a, b);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let one = builder.ins().iconst(types::I64, 1);
                        stack.push(builder.ins().select(cmp, one, zero));
                    }
                    SpecializedOpcode::IntLe => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, a, b);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let one = builder.ins().iconst(types::I64, 1);
                        stack.push(builder.ins().select(cmp, one, zero));
                    }
                    SpecializedOpcode::IntGt => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, a, b);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let one = builder.ins().iconst(types::I64, 1);
                        stack.push(builder.ins().select(cmp, one, zero));
                    }
                    SpecializedOpcode::IntGe => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, a, b);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let one = builder.ins().iconst(types::I64, 1);
                        stack.push(builder.ins().select(cmp, one, zero));
                    }
                    SpecializedOpcode::IntEq => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        let cmp = builder.ins().icmp(IntCC::Equal, a, b);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let one = builder.ins().iconst(types::I64, 1);
                        stack.push(builder.ins().select(cmp, one, zero));
                    }
                    SpecializedOpcode::IntNe => {
                        let b = stack.pop().ok_or("Stack underflow")?;
                        let a = stack.pop().ok_or("Stack underflow")?;
                        let cmp = builder.ins().icmp(IntCC::NotEqual, a, b);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let one = builder.ins().iconst(types::I64, 1);
                        stack.push(builder.ins().select(cmp, one, zero));
                    }
                    SpecializedOpcode::Generic(Opcode::Jmp(label)) => {
                        let target = cl_blocks.get(label).ok_or_else(|| format!("Unknown label {}", label))?;
                        builder.ins().jump(*target, &[]);
                        terminated = true;
                    }
                    SpecializedOpcode::Generic(Opcode::JmpIfFalse(label)) => {
                        let cond = stack.pop().ok_or("Stack underflow")?;
                        let target = cl_blocks.get(label).ok_or_else(|| format!("Unknown label {}", label))?;
                        
                        let next_func_block = func_def.blocks.get(i + 1).ok_or("JmpIfFalse must have fallthrough")?;
                        let next_cl_block = cl_blocks[&next_func_block.label];
                        
                        builder.ins().brif(cond, next_cl_block, &[], *target, &[]);
                        terminated = true;
                    }
                    SpecializedOpcode::Generic(Opcode::Ret) => {
                        let res = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
                        builder.ins().return_(&[res]);
                        terminated = true;
                    }
                    _ => return Err(format!("JIT does not support opcode {:?}", instr.opcode)),
                }
            }
            
            if !terminated {
                 if let Some(next_func_block) = func_def.blocks.get(i + 1) {
                    let next_cl_block = cl_blocks[&next_func_block.label];
                    builder.ins().jump(next_cl_block, &[]);
                } else {
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero]);
                }
            }
        }
        
        // Seal all blocks AFTER building them all to handle back-edges correctly
        for (_, cl_block) in cl_blocks {
            builder.seal_block(cl_block);
        }

        builder.finalize();

        let id = self.module
            .declare_function(&func_def.name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        self.module.finalize_definitions().map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(id);
        Ok(code)
    }
}
