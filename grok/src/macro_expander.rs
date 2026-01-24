use crate::ast::{AstNode, Pattern, Span};
use std::collections::HashMap;

pub struct MacroExpander {
    macros: HashMap<String, Vec<(Pattern, AstNode)>>,
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    pub fn expand(&mut self, ast: AstNode) -> AstNode {
        match ast {
            AstNode::Program(nodes) => {
                let mut new_nodes = Vec::new();
                for node in nodes {
                    let expanded = self.expand(node);
                    // Filter out empty blocks which represent consumed MacroDefs
                    if let AstNode::Block(ref stmts) = expanded {
                        if stmts.is_empty() {
                            continue;
                        }
                    }
                    new_nodes.push(expanded);
                }
                AstNode::Program(new_nodes)
            }
            AstNode::Block(nodes) => {
                let mut new_nodes = Vec::new();
                for node in nodes {
                    new_nodes.push(self.expand(node));
                }
                AstNode::Block(new_nodes)
            }
            AstNode::MacroDef { name, rules, .. } => {
                self.macros.insert(name, rules);
                AstNode::Block(vec![]) // Marker for removal
            }
            AstNode::MacroCall { name, args, span } => {
                let expanded_args: Vec<AstNode> =
                    args.into_iter().map(|a| self.expand(a)).collect();
                if let Some(rules) = self.macros.get(&name).cloned() {
                    for (pattern, template) in rules {
                        let mut bindings = HashMap::new();
                        if self.match_pattern_top(&pattern, &expanded_args, &mut bindings) {
                            return self.expand(self.substitute(template, &bindings));
                        }
                    }
                }
                AstNode::MacroCall {
                    name,
                    args: expanded_args,
                    span,
                }
            }
            AstNode::FunctionDef {
                name,
                params,
                return_type,
                body,
                decorators,
                span,
            } => AstNode::FunctionDef {
                name,
                params,
                return_type,
                body: Box::new(self.expand(*body)),
                decorators,
                span,
            },
            AstNode::LetStmt {
                name,
                mutable,
                ty,
                expr,
                span,
            } => AstNode::LetStmt {
                name,
                mutable,
                ty,
                expr: Box::new(self.expand(*expr)),
                span,
            },
            AstNode::IfExpr {
                condition,
                then_body,
                else_body,
                span,
            } => AstNode::IfExpr {
                condition: Box::new(self.expand(*condition)),
                then_body: Box::new(self.expand(*then_body)),
                else_body: else_body.map(|b| Box::new(self.expand(*b))),
                span,
            },
            AstNode::WhileLoop {
                condition,
                body,
                span,
            } => AstNode::WhileLoop {
                condition: Box::new(self.expand(*condition)),
                body: Box::new(self.expand(*body)),
                span,
            },
            AstNode::ForLoop {
                var,
                iterable,
                body,
                span,
            } => AstNode::ForLoop {
                var,
                iterable: Box::new(self.expand(*iterable)),
                body: Box::new(self.expand(*body)),
                span,
            },
            AstNode::BinaryOp {
                left,
                op,
                right,
                span,
            } => AstNode::BinaryOp {
                left: Box::new(self.expand(*left)),
                op,
                right: Box::new(self.expand(*right)),
                span,
            },
            AstNode::FunctionCall { func, args, span } => AstNode::FunctionCall {
                func: Box::new(self.expand(*func)),
                args: args.into_iter().map(|a| self.expand(a)).collect(),
                span,
            },
            AstNode::MatchExpr {
                scrutinee,
                arms,
                span,
            } => AstNode::MatchExpr {
                scrutinee: Box::new(self.expand(*scrutinee)),
                arms: arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.body = self.expand(arm.body);
                        arm
                    })
                    .collect(),
                span,
            },
            AstNode::StructLiteral { name, fields, span } => AstNode::StructLiteral {
                name,
                fields: fields
                    .into_iter()
                    .map(|(n, e)| (n, self.expand(e)))
                    .collect(),
                span,
            },
            AstNode::MemberAccess {
                object,
                member,
                span,
            } => AstNode::MemberAccess {
                object: Box::new(self.expand(*object)),
                member,
                span,
            },
            _ => ast,
        }
    }

    fn match_pattern_top(
        &self,
        pattern: &Pattern,
        args: &[AstNode],
        bindings: &mut HashMap<String, AstNode>,
    ) -> bool {
        match pattern {
            Pattern::Tuple(patterns) => {
                if patterns.len() == args.len() {
                    for (p, a) in patterns.iter().zip(args.iter()) {
                        if !self.match_single_pattern(p, a, bindings) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            _ => {
                if args.len() == 1 {
                    self.match_single_pattern(pattern, &args[0], bindings)
                } else {
                    false
                }
            }
        }
    }

    fn match_single_pattern(
        &self,
        pattern: &Pattern,
        arg: &AstNode,
        bindings: &mut HashMap<String, AstNode>,
    ) -> bool {
        match (pattern, arg) {
            (Pattern::Identifier(name), _) => {
                bindings.insert(name.clone(), arg.clone());
                true
            }
            (Pattern::IntLiteral(p), AstNode::IntLiteral(a, _)) => p == a,
            (Pattern::BoolLiteral(p), AstNode::BoolLiteral(a, _)) => p == a,
            (Pattern::StringLiteral(p), AstNode::StringLiteral(a, _)) => p == a,
            (Pattern::Underscore, _) => true,
            _ => false,
        }
    }

    fn substitute(&self, template: AstNode, bindings: &HashMap<String, AstNode>) -> AstNode {
        match template {
            AstNode::Identifier(name, span) => {
                if let Some(bound) = bindings.get(&name) {
                    bound.clone()
                } else {
                    AstNode::Identifier(name, span)
                }
            }
            AstNode::Block(nodes) => AstNode::Block(
                nodes
                    .into_iter()
                    .map(|n| self.substitute(n, bindings))
                    .collect(),
            ),
            AstNode::FunctionDef {
                name,
                params,
                return_type,
                body,
                decorators,
                span,
            } => AstNode::FunctionDef {
                name,
                params,
                return_type,
                body: Box::new(self.substitute(*body, bindings)),
                decorators,
                span,
            },
            AstNode::LetStmt {
                name,
                mutable,
                ty,
                expr,
                span,
            } => AstNode::LetStmt {
                name,
                mutable,
                ty,
                expr: Box::new(self.substitute(*expr, bindings)),
                span,
            },
            AstNode::IfExpr {
                condition,
                then_body,
                else_body,
                span,
            } => AstNode::IfExpr {
                condition: Box::new(self.substitute(*condition, bindings)),
                then_body: Box::new(self.substitute(*then_body, bindings)),
                else_body: else_body.map(|b| Box::new(self.substitute(*b, bindings))),
                span,
            },
            AstNode::WhileLoop {
                condition,
                body,
                span,
            } => AstNode::WhileLoop {
                condition: Box::new(self.substitute(*condition, bindings)),
                body: Box::new(self.substitute(*body, bindings)),
                span,
            },
            AstNode::ForLoop {
                var,
                iterable,
                body,
                span,
            } => AstNode::ForLoop {
                var,
                iterable: Box::new(self.substitute(*iterable, bindings)),
                body: Box::new(self.substitute(*body, bindings)),
                span,
            },
            AstNode::BinaryOp {
                left,
                op,
                right,
                span,
            } => AstNode::BinaryOp {
                left: Box::new(self.substitute(*left, bindings)),
                op,
                right: Box::new(self.substitute(*right, bindings)),
                span,
            },
            AstNode::FunctionCall { func, args, span } => AstNode::FunctionCall {
                func: Box::new(self.substitute(*func, bindings)),
                args: args
                    .into_iter()
                    .map(|a| self.substitute(a, bindings))
                    .collect(),
                span,
            },
            AstNode::MacroCall { name, args, span } => AstNode::MacroCall {
                name,
                args: args
                    .into_iter()
                    .map(|a| self.substitute(a, bindings))
                    .collect(),
                span,
            },
            AstNode::MatchExpr {
                scrutinee,
                arms,
                span,
            } => AstNode::MatchExpr {
                scrutinee: Box::new(self.substitute(*scrutinee, bindings)),
                arms: arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.body = self.substitute(arm.body, bindings);
                        arm
                    })
                    .collect(),
                span,
            },
            AstNode::StructLiteral { name, fields, span } => AstNode::StructLiteral {
                name,
                fields: fields
                    .into_iter()
                    .map(|(n, e)| (n, self.substitute(e, bindings)))
                    .collect(),
                span,
            },
            AstNode::MemberAccess {
                object,
                member,
                span,
            } => AstNode::MemberAccess {
                object: Box::new(self.substitute(*object, bindings)),
                member,
                span,
            },
            _ => template,
        }
    }
}
