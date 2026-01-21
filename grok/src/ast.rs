use typed_arena::Arena;

#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<AstNode>),
    Function {
        name: String,
        params: Vec<Param>,
        body: Box<AstNode>,
        return_type: Option<Type>,
    },
    Let {
        name: String,
        mutable: bool,
        expr: Box<AstNode>,
    },
    If {
        condition: Box<AstNode>,
        then_body: Box<AstNode>,
        else_body: Option<Box<AstNode>>,
    },
    // Add more as needed
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int32,
    Float64,
    Bool,
    String,
    // Add more
}
