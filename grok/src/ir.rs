// grok/src/ir.rs
use crate::ast::AstNode;

#[derive(Debug, Clone)]
pub enum Opcode {
    PushInt(i64),
    PushFloat(f64),
    PushStr(String),
    PushBool(bool),
    LoadVar(String),
    StoreVar(String),
    LoadField(String),
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Jmp(String),
    JmpIfFalse(String),
    PushStruct(String, Vec<String>),
    Call(String, usize),
    Ret,
    Spawn(String, usize),
    Send,
    Receive,
}

#[derive(Debug, Clone)]
pub struct IRInstruction {
    pub opcode: Opcode,
}

#[derive(Debug, Clone)]
pub struct IRBlock {
    pub label: String,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<String>,
    pub blocks: Vec<IRBlock>,
}

pub struct IRGenerator {
    temp_counter: usize,
}

impl IRGenerator {
    pub fn new() -> Self {
        Self { temp_counter: 0 }
    }

    pub fn generate(&mut self, ast: &AstNode) -> Vec<IRFunction> {
        let mut functions = Vec::new();
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    functions.extend(self.generate(node));
                }
            }
            AstNode::FunctionDef {
                name, params, body, ..
            } => {
                let mut blocks = vec![IRBlock {
                    label: "entry".to_string(),
                    instructions: Vec::new(),
                }];
                self.gen_expr(body, &mut blocks);

                // Ensure return
                if let Some(last_block) = blocks.last_mut() {
                    if last_block.instructions.is_empty()
                        || !matches!(last_block.instructions.last().unwrap().opcode, Opcode::Ret)
                    {
                        last_block.instructions.push(IRInstruction {
                            opcode: Opcode::Ret,
                        });
                    }
                }

                functions.push(IRFunction {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    blocks,
                });
            }
            AstNode::ActorDef { name, body, .. } => {
                let mut blocks = vec![IRBlock {
                    label: "entry".to_string(),
                    instructions: Vec::new(),
                }];
                self.gen_expr(body, &mut blocks);

                // Ensure return
                if let Some(last_block) = blocks.last_mut() {
                    if last_block.instructions.is_empty()
                        || !matches!(last_block.instructions.last().unwrap().opcode, Opcode::Ret)
                    {
                        last_block.instructions.push(IRInstruction {
                            opcode: Opcode::Ret,
                        });
                    }
                }

                functions.push(IRFunction {
                    name: name.clone(),
                    params: vec![],
                    blocks,
                });
            }
            _ => {}
        }
        functions
    }

    fn gen_expr(&mut self, expr: &AstNode, blocks: &mut Vec<IRBlock>) {
        let current_block = blocks.last_mut().expect("No blocks in IR generation");
        match expr {
            AstNode::IntLiteral(val, _) => {
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::PushInt(*val),
                });
            }
            AstNode::FloatLiteral(val, _) => {
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::PushFloat(*val),
                });
            }
            AstNode::StringLiteral(val, _) => {
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::PushStr(val.clone()),
                });
            }
            AstNode::BoolLiteral(val, _) => {
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::PushBool(*val),
                });
            }
            AstNode::Identifier(name, _) => {
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::LoadVar(name.clone()),
                });
            }
            AstNode::LetStmt { name, expr, .. } => {
                self.gen_expr(expr, blocks);
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::StoreVar(name.clone()),
                });
            }
            AstNode::BinaryOp {
                left, op, right, ..
            } => {
                self.gen_expr(left, blocks);
                self.gen_expr(right, blocks);
                let current_block = blocks.last_mut().unwrap();
                let opcode = match op.as_str() {
                    "+" => Opcode::Add,
                    "-" => Opcode::Sub,
                    "*" => Opcode::Mul,
                    "/" => Opcode::Div,
                    "==" => Opcode::Eq,
                    "!=" => Opcode::Ne,
                    "<" => Opcode::Lt,
                    ">" => Opcode::Gt,
                    "<=" => Opcode::Le,
                    ">=" => Opcode::Ge,
                    _ => Opcode::Ret, // Should be error
                };
                current_block.instructions.push(IRInstruction { opcode });
            }
            AstNode::Return { value, .. } => {
                if let Some(v) = value {
                    self.gen_expr(v, blocks);
                }
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::Ret,
                });
            }
            AstNode::Block(stmts) => {
                for stmt in stmts {
                    self.gen_expr(stmt, blocks);
                }
            }
            AstNode::IfExpr {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let id = self.temp_counter;
                self.temp_counter += 1;
                let then_label = format!("then_{}", id);
                let else_label = format!("else_{}", id);
                let end_label = format!("end_{}", id);

                self.gen_expr(condition, blocks);
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::JmpIfFalse(else_label.clone()),
                });

                // Then
                blocks.push(IRBlock {
                    label: then_label,
                    instructions: Vec::new(),
                });
                self.gen_expr(then_body, blocks);
                blocks.last_mut().unwrap().instructions.push(IRInstruction {
                    opcode: Opcode::Jmp(end_label.clone()),
                });

                // Else
                blocks.push(IRBlock {
                    label: else_label,
                    instructions: Vec::new(),
                });
                if let Some(e) = else_body {
                    self.gen_expr(e, blocks);
                }
                blocks.last_mut().unwrap().instructions.push(IRInstruction {
                    opcode: Opcode::Jmp(end_label.clone()),
                });

                // End
                blocks.push(IRBlock {
                    label: end_label,
                    instructions: Vec::new(),
                });
            }
            AstNode::WhileLoop {
                condition, body, ..
            } => {
                let id = self.temp_counter;
                self.temp_counter += 1;
                let cond_label = format!("while_cond_{}", id);
                let body_label = format!("while_body_{}", id);
                let end_label = format!("while_end_{}", id);

                // Jump to condition
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::Jmp(cond_label.clone()),
                });

                // Condition
                blocks.push(IRBlock {
                    label: cond_label.clone(),
                    instructions: Vec::new(),
                });
                self.gen_expr(condition, blocks);
                blocks.last_mut().unwrap().instructions.push(IRInstruction {
                    opcode: Opcode::JmpIfFalse(end_label.clone()),
                });

                // Body
                blocks.push(IRBlock {
                    label: body_label,
                    instructions: Vec::new(),
                });
                self.gen_expr(body, blocks);
                blocks.last_mut().unwrap().instructions.push(IRInstruction {
                    opcode: Opcode::Jmp(cond_label),
                });

                // End
                blocks.push(IRBlock {
                    label: end_label,
                    instructions: Vec::new(),
                });
            }
            AstNode::Loop { body, .. } => {
                let id = self.temp_counter;
                self.temp_counter += 1;
                let body_label = format!("loop_body_{}", id);
                let end_label = format!("loop_end_{}", id);

                // Jump to body
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::Jmp(body_label.clone()),
                });

                // Body
                blocks.push(IRBlock {
                    label: body_label.clone(),
                    instructions: Vec::new(),
                });
                self.gen_expr(body, blocks);
                blocks.last_mut().unwrap().instructions.push(IRInstruction {
                    opcode: Opcode::Jmp(body_label),
                });

                // End
                blocks.push(IRBlock {
                    label: end_label,
                    instructions: Vec::new(),
                });
            }
            AstNode::MatchExpr {
                scrutinee, arms, ..
            } => {
                self.gen_expr(scrutinee, blocks);
                for arm in arms {
                    self.gen_arm(arm, blocks);
                }
            }
            AstNode::StructLiteral { name, fields, .. } => {
                let mut f_names = Vec::new();
                for (f_name, f_expr) in fields {
                    self.gen_expr(f_expr, blocks);
                    f_names.push(f_name.clone());
                }
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::PushStruct(name.clone(), f_names),
                });
            }
            AstNode::FunctionCall { func, args, .. } => {
                if let AstNode::Identifier(name, _) = &**func {
                    for arg in args {
                        self.gen_expr(arg, blocks);
                    }
                    let current_block = blocks.last_mut().unwrap();
                    current_block.instructions.push(IRInstruction {
                        opcode: Opcode::Call(name.clone(), args.len()),
                    });
                }
            }
            AstNode::Spawn { actor, args, .. } => {
                for (_, arg) in args {
                    self.gen_expr(arg, blocks);
                }
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::Spawn(actor.clone(), args.len()),
                });
            }
            AstNode::Send {
                target, message, ..
            } => {
                self.gen_expr(target, blocks);
                self.gen_expr(message, blocks);
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::Send,
                });
            }
            AstNode::Receive { arms, .. } => {
                // Simplified receive: just wait for any message
                // This will push the message to the stack
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::Receive,
                });
                for arm in arms {
                    self.gen_arm(arm, blocks);
                }
            }
            AstNode::MemberAccess { object, member, .. } => {
                self.gen_expr(object, blocks);
                let current_block = blocks.last_mut().unwrap();
                current_block.instructions.push(IRInstruction {
                    opcode: Opcode::LoadField(member.clone()),
                });
            }
            _ => {}
        }
    }

    fn gen_arm(&mut self, arm: &crate::ast::MatchArm, blocks: &mut Vec<IRBlock>) {
        // Skeletal implementation of arm IR generation
        self.gen_expr(&arm.body, blocks);
    }
}
