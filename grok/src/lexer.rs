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
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("loop")]
    Loop,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("_", priority = 2)]
    Underscore,

    // Literals
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,
    #[regex(r#"'([^'\\]|\\.)'"#)]
    Char,
    #[regex(r#"b"([^"\\]|\\.)*""#)]
    ByteString,
    #[regex(r"[0-9]+")]
    Int,
    #[regex(r"[0-9]+\.[0-9]+", priority = 2)]
    Float,

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

pub struct TokenData {
    pub token: Token,
    pub slice: String,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
    line: usize,
    last_newline: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
            line: 1,
            last_newline: 0,
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<TokenData, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(token)) => {
                let slice = self.inner.slice();
                let span = self.inner.span();
                
                // Update line and column
                let start = span.start;
                let col = start - self.last_newline + 1;
                
                // If the slice contains newlines, update our tracking for the NEXT token
                for (i, c) in slice.char_indices() {
                    if c == '\n' {
                        self.line += 1;
                        self.last_newline = start + i + 1;
                    }
                }

                Some(Ok(TokenData {
                    token,
                    slice: slice.to_string(),
                    line: self.line,
                    col,
                }))
            }
            Some(Err(_)) => Some(Err(())),
            None => None,
        }
    }
}
