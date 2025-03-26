use logos::{Logos, SpannedIter};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Deserialize, Logos, Serialize)]
#[repr(C)]
pub enum TokenKind {
    #[regex(r"[\s]+")]
    Whitespace,
    #[regex(r"#.*")]
    Comment,
    #[regex(r"[\p{L}_][\p{L}\p{N}_]*")]
    Ident,

    // Literals ===========================================================================
    #[regex(r"(\d[\d_]*(\._*\d[\d_]*)?(e_*[+-]?_*\d[\d_]*)?)")]
    Number,

    // Keywords ===========================================================================
    #[token("def")]
    Def,
    #[token("else")]
    Else,
    #[token("extern")]
    Extern,
    #[token("for")]
    For,
    #[token("if")]
    If,
    #[token("then")]
    Then,

    // Operators ==========================================================================
    #[token("=")]
    Assign,

    // Comparison Operators
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,

    // Arithmetic Operators
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,

    // Boolean Operators
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    // Punctuation ========================================================================
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token(";")]
    Semicolon,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Whitespace => f.write_str("Whitespace"),
            TokenKind::Comment => f.write_str("Comment"),
            TokenKind::Ident => f.write_str("Ident"),
            TokenKind::Number => f.write_str("Number"),

            TokenKind::Def
            | TokenKind::Else
            | TokenKind::Extern
            | TokenKind::For
            | TokenKind::If
            | TokenKind::Then => f.write_str("Keyword"),

            TokenKind::Assign
            | TokenKind::Eq
            | TokenKind::Ne
            | TokenKind::Gt
            | TokenKind::Ge
            | TokenKind::Lt
            | TokenKind::Le
            | TokenKind::Add
            | TokenKind::Sub
            | TokenKind::Mul
            | TokenKind::Div
            | TokenKind::Mod
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Not => f.write_str("Operator"),

            TokenKind::OpenParen | TokenKind::CloseParen | TokenKind::Semicolon => {
                f.write_str("Punctuation")
            }
        }
    }
}

#[derive(Debug)]
pub enum Source {
    Stdin(String),
    String(String),
    File { path: PathBuf, contents: String },
}

impl Source {
    pub fn text(&self) -> &str {
        match self {
            Self::Stdin(s) => s,
            Self::String(s) => s,
            Self::File { contents, .. } => contents,
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin(_) => f.write_str("stdin"),
            Self::String(_) => f.write_str("string"),
            Self::File { path, .. } => write!(f, "{}", path.display()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DebugSpan {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Display for DebugSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}..{}", self.line, self.start, self.end)
    }
}

#[derive(Debug)]
pub struct Token<'s> {
    pub kind: TokenKind,
    pub text: &'s str,
    pub span: DebugSpan,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self.kind, TokenKind::Whitespace | TokenKind::Comment) {
            write!(f, "({})", self.kind)
        } else {
            write!(f, "({}, `{}`)", self.kind, self.text)
        }
    }
}

pub struct Lexer<'s> {
    source: &'s Source,
    iter: SpannedIter<'s, TokenKind>,
    line: usize,
    symbols: HashMap<&'s str, Vec<DebugSpan>>,
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Result<Token<'s>, DebugSpan>;
    fn next(&mut self) -> Option<Self::Item> {
        let (kind_res, span) = self.iter.next()?;
        let span = DebugSpan {
            line: self.line,
            start: span.start,
            end: span.end,
        };

        if let Ok(kind) = kind_res {
            let text = &self.source.text()[span.start..span.end];
            self.line += text.chars().filter(|c| *c == '\n').count();

            if matches!(kind, TokenKind::Ident) {
                if let Some(spans) = self.symbols.get_mut(text) {
                    spans.push(span);
                } else {
                    self.symbols.insert(text, vec![span]);
                }
            }

            Some(Ok(Token { kind, text, span }))
        } else {
            Some(Err(span))
        }
    }
}

impl<'s> Lexer<'s> {
    pub fn new(source: &'s Source) -> Self {
        let text = source.text();
        let iter = TokenKind::lexer(text).spanned();
        let line = 0;
        let symbols = HashMap::new();
        Self {
            source,
            iter,
            line,
            symbols,
        }
    }

    pub fn finish(self) -> HashMap<&'s str, Vec<DebugSpan>> {
        self.symbols
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ident {
    pub index: usize,
    pub span: DebugSpan,
}

#[derive(Debug, Clone, Copy)]
pub struct Literal {
    pub val: f64,
    pub span: DebugSpan,
}

macro_rules! token_kind_as {
    ($n: ident: $($k: ident),*) => {
        #[derive(Debug, Clone, Copy)]
        pub enum $n { $($k = TokenKind::$k as isize),* }

        impl TryFrom<TokenKind> for $n {
            type Error = TokenKind;
            fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
                match value {
                    $(TokenKind::$k => Ok($n::$k),)*
                    _ => Err(value),
                }
            }
        }

        impl From<$n> for TokenKind {
            fn from(value: $n) -> Self {
                match value { $($n::$k => TokenKind::$k),* }
            }
        }
    };
}

token_kind_as!(KeywordKind: Def, Else, Extern, For, If, Then);

#[derive(Debug, Clone, Copy)]
pub struct Keyword {
    pub kind: KeywordKind,
    pub span: DebugSpan,
}

token_kind_as! {
    OperatorKind:
        Assign,
        Eq, Ne, Gt, Ge, Lt, Le,
        Add, Sub, Mul, Div, Mod,
        And, Or, Not
}

#[derive(Clone, Copy, Debug)]
pub struct Operator {
    pub kind: OperatorKind,
    pub span: DebugSpan,
}

token_kind_as!(PunctuationKind: OpenParen, CloseParen, Semicolon);

impl Operator {
    #[inline]
    pub const fn is_unary(&self) -> bool {
        matches!(self.kind, OperatorKind::Sub | OperatorKind::Not)
    }

    #[inline]
    pub const fn is_binary(&self) -> bool {
        !self.is_unary()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Punctuation {
    pub kind: PunctuationKind,
    pub span: DebugSpan,
}
