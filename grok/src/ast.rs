#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Program(Vec<AstNode>),
    
    // Declarations
    FunctionDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Box<AstNode>, // Block
        decorators: Vec<String>,
        span: Span,
    },
    StructDef {
        name: String,
        fields: Vec<(String, Type)>,
        generics: Vec<String>,
        span: Span,
    },
    EnumDef {
        name: String,
        variants: Vec<(String, Option<Type>)>,
        generics: Vec<String>,
        span: Span,
    },
    TraitDef {
        name: String,
        methods: Vec<AstNode>, // FunctionDef placeholders
        bounds: Vec<String>,
        span: Span,
    },
    ActorDef {
        name: String,
        body: Box<AstNode>, // Block
        span: Span,
    },
    Spawn {
        actor: String,
        args: Vec<(String, AstNode)>,
        span: Span,
    },
    Send {
        target: Box<AstNode>,
        message: Box<AstNode>,
        span: Span,
    },
    Receive {
        arms: Vec<MatchArm>,
        span: Span,
    },

    // Statements
    LetStmt {
        name: String,
        mutable: bool,
        ty: Option<Type>,
        expr: Box<AstNode>,
        span: Span,
    },
    Return {
        value: Option<Box<AstNode>>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Loop {
        body: Box<AstNode>, // Block
        span: Span,
    },
    WhileLoop {
        condition: Box<AstNode>,
        body: Box<AstNode>,
        span: Span,
    },
    ForLoop {
        var: String,
        iterable: Box<AstNode>,
        body: Box<AstNode>,
        span: Span,
    },
    Block(Vec<AstNode>),

    // Expressions
    BinaryOp {
        left: Box<AstNode>,
        op: String,
        right: Box<AstNode>,
        span: Span,
    },
    UnaryOp {
        op: String,
        operand: Box<AstNode>,
        span: Span,
    },
    FunctionCall {
        func: Box<AstNode>,
        args: Vec<AstNode>,
        span: Span,
    },
    MemberAccess {
        object: Box<AstNode>,
        member: String,
        span: Span,
    },
    MatchExpr {
        scrutinee: Box<AstNode>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    IfExpr {
        condition: Box<AstNode>,
        then_body: Box<AstNode>,
        else_body: Option<Box<AstNode>>,
        span: Span,
    },
    Identifier(String, Span),
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    StringLiteral(String, Span),
    ByteStringLiteral(Vec<u8>, Span),
    MacroDef {
        name: String,
        rules: Vec<(Pattern, AstNode)>,
        span: Span,
    },
    MacroCall {
        name: String,
        args: Vec<AstNode>,
        span: Span,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, AstNode)>,
        span: Span,
    },
    BoolLiteral(bool, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Underscore,
    Tuple(Vec<Pattern>),
    Struct(String, Vec<(String, Pattern)>),
    Enum(String, String, Option<Box<Pattern>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<AstNode>,
    pub body: AstNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(String),
    Variable(String),
    Generic(String, Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String, Vec<(String, Type)>),
    Trait(String),
    Reference(Box<Type>, bool),
    Actor(String),
    Unit,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Primitive(s) => write!(f, "{}", s),
            Type::Variable(s) => write!(f, "{}", s),
            Type::Generic(n, args) => {
                write!(f, "{}<", n)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
            Type::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Struct(n, _) => write!(f, "struct {}", n),
            Type::Trait(n) => write!(f, "trait {}", n),
            Type::Reference(t, mutable) => write!(f, "&{}{}", if *mutable { "mut " } else { "" }, t),
            Type::Actor(n) => write!(f, "actor {}", n),
            Type::Unit => write!(f, "()"),
        }
    }
}
