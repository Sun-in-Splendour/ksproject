use logos::{Logos, SpannedIter};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Deserialize, Logos, Serialize, PartialEq, Eq)]
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
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
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
    #[token("return")]
    Return,
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
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
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

            TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Def
            | TokenKind::Else
            | TokenKind::Extern
            | TokenKind::For
            | TokenKind::If
            | TokenKind::Return
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

            TokenKind::OpenParen
            | TokenKind::CloseParen
            | TokenKind::OpenBrace
            | TokenKind::CloseBrace
            | TokenKind::Semicolon => f.write_str("Punctuation"),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
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

impl DebugSpan {
    pub fn merge(self, other: Self) -> Self {
        DebugSpan {
            line: self.line,
            start: self.start,
            end: other.end,
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct TokenJson {
    pub kind: TokenKind,
    pub span: DebugSpan,
}

impl From<Token<'_>> for TokenJson {
    fn from(token: Token) -> Self {
        Self {
            kind: token.kind,
            span: token.span,
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
pub enum Operator {
    Assign = TokenKind::Assign as isize,

    // Comparison Operators
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,

    // Arithmetic Operators
    Add,
    Sub, // ALSO Neg
    Mul,
    Div,
    Mod,

    // Boolean Operators
    And,
    Or,
    Not = TokenKind::Not as isize,
}

impl TryFrom<TokenKind> for Operator {
    type Error = ();
    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Assign => Ok(Self::Assign),
            TokenKind::Eq => Ok(Self::Eq),
            TokenKind::Ne => Ok(Self::Ne),
            TokenKind::Gt => Ok(Self::Gt),
            TokenKind::Ge => Ok(Self::Ge),
            TokenKind::Lt => Ok(Self::Lt),
            TokenKind::Le => Ok(Self::Le),
            TokenKind::Add => Ok(Self::Add),
            TokenKind::Sub => Ok(Self::Sub),
            TokenKind::Mul => Ok(Self::Mul),
            TokenKind::Div => Ok(Self::Div),
            TokenKind::Mod => Ok(Self::Mod),
            TokenKind::And => Ok(Self::And),
            TokenKind::Or => Ok(Self::Or),
            TokenKind::Not => Ok(Self::Not),
            _ => Err(()),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign => f.write_str("="),
            Self::Eq => f.write_str("=="),
            Self::Ne => f.write_str("!="),
            Self::Gt => f.write_str(">"),
            Self::Ge => f.write_str(">="),
            Self::Lt => f.write_str("<"),
            Self::Le => f.write_str("<="),
            Self::Add => f.write_str("+"),
            Self::Sub => f.write_str("-"),
            Self::Mul => f.write_str("*"),
            Self::Div => f.write_str("/"),
            Self::Mod => f.write_str("%"),
            Self::And => f.write_str("&&"),
            Self::Or => f.write_str("||"),
            Self::Not => f.write_str("!"),
        }
    }
}
