use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("actor")]
    Actor,
    #[token("spawn")]
    Spawn,
    #[token("send")]
    Send,
    #[token("receive")]
    Receive,
    #[token("module")]
    Module,
    #[token("macro")]
    Macro,
    #[token("macro_rules")]
    MacroRules,
    #[token("ai")]
    Ai,
    #[token("test")]
    Test,
    #[token("optimize")]
    Optimize,
    #[token("pub")]
    Pub,
    #[token("mut")]
    Mut,
    #[token("const")]
    Const,
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("from")]
    From,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("use")]
    Use,
    #[token("mod")]
    Mod,
    #[token("self")]
    Self_,
    #[token("Self")]
    SelfType,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("where")]
    Where,
    #[token("type")]
    Type,
    #[token("unsafe")]
    Unsafe,
    #[token("extern")]
    Extern,
    #[token("static")]
    Static,
    #[token("move")]
    Move,
    #[token("in")]
    In,
    #[token("_", priority = 2)]
    Underscore,

    // Literals
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,
    #[regex(r#"'([^'\\]|\\.)'"#)]
    Char,
    #[regex(r"[0-9]+")]
    Int,
    #[regex(r"[0-9]+\\.[0-9]+")]
    Float,
    // RawString, ByteString - TODO
    // #[regex(r"r\"([^\"])*\"")]
    // RawString,
    // #[regex(r"b\"([^\"\\\\]|\\\\.)*\"")]
    // ByteString,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Assign,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Bang,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
    #[token("?")]
    Question,
    #[token("@")]
    At,
    #[token("#")]
    Hash,
    #[token("$")]
    Dollar,
    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(".")]
    Dot,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 1)]
    Identifier,
}

pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
