use logos::{Logos, SpannedIter};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[repr(C)]
pub enum TokenKind {
    #[token("\u{FEFF}")]
    UTF8BOM,
    #[regex(r"\s+")]
    Whitespace,
    #[regex(r"#[^\n]*(\n)?")]
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
    #[token("in")]
    In,
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
    #[token(",")]
    Comma,
}

impl TokenKind {
    const fn kind_str(self) -> &'static str {
        match self {
            TokenKind::UTF8BOM => "UTF8BOM",
            TokenKind::Whitespace => "Whitespace",
            TokenKind::Comment => "Comment",
            TokenKind::Ident => "Ident",
            TokenKind::Number => "Number",

            TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Def
            | TokenKind::Else
            | TokenKind::Extern
            | TokenKind::For
            | TokenKind::If
            | TokenKind::In
            | TokenKind::Return
            | TokenKind::Then => "Keyword",

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
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Not => "Operator",

            TokenKind::OpenParen
            | TokenKind::CloseParen
            | TokenKind::OpenBrace
            | TokenKind::CloseBrace
            | TokenKind::Semicolon
            | TokenKind::Comma => "Punctuation",
        }
    }

    const fn value_str(self) -> Option<&'static str> {
        match self {
            TokenKind::UTF8BOM
            | TokenKind::Whitespace
            | TokenKind::Comment
            | TokenKind::Ident
            | TokenKind::Number => None,

            TokenKind::Break => Some("break"),
            TokenKind::Continue => Some("continue"),
            TokenKind::Def => Some("def"),
            TokenKind::Else => Some("else"),
            TokenKind::Extern => Some("extern"),
            TokenKind::For => Some("for"),
            TokenKind::If => Some("if"),
            TokenKind::In => Some("in"),
            TokenKind::Return => Some("return"),
            TokenKind::Then => Some("then"),

            TokenKind::Assign => Some("="),

            TokenKind::Eq => Some("=="),
            TokenKind::Ne => Some("!="),
            TokenKind::Gt => Some(">"),
            TokenKind::Ge => Some(">="),
            TokenKind::Lt => Some("<"),
            TokenKind::Le => Some("<="),

            TokenKind::Add => Some("+"),
            TokenKind::Sub => Some("-"),
            TokenKind::Mul => Some("*"),
            TokenKind::Div => Some("/"),

            TokenKind::And => Some("&&"),
            TokenKind::Or => Some("||"),
            TokenKind::Not => Some("!"),

            TokenKind::OpenParen => Some("("),
            TokenKind::CloseParen => Some(")"),
            TokenKind::OpenBrace => Some("{"),
            TokenKind::CloseBrace => Some("}"),
            TokenKind::Semicolon => Some(";"),
            TokenKind::Comma => Some(","),
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

#[derive(Default)]
pub struct SourceSequence {
    pub sources: Vec<Source>,
}

impl SourceSequence {
    pub fn new() -> Self {
        Self { sources: vec![] }
    }

    pub fn add(&mut self, source: Source) -> usize {
        let index = self.sources.len();
        self.sources.push(source);
        index
    }

    pub fn get_text(&self, span: CodeSpan) -> &str {
        &self.sources[span.src_id].text()[span.start..span.end]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[repr(C)]
pub struct CodeSpan {
    pub line: usize,

    pub src_id: usize,
    pub start: usize,
    pub end: usize,
}

impl Display for CodeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}..{}", self.line, self.start, self.end)
    }
}

impl CodeSpan {
    pub fn merge(self, other: Self) -> Self {
        CodeSpan {
            src_id: self.src_id,
            line: self.line,
            start: self.start,
            end: other.end,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: CodeSpan,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.kind.value_str() {
            write!(f, "<{}, `{}`>", self.kind.kind_str(), value)
        } else {
            write!(f, "<{}>", self.kind.kind_str())
        }
    }
}

pub struct Lexer<'s> {
    src_id: usize,
    text: &'s str,
    iter: SpannedIter<'s, TokenKind>,
    line: usize,
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, CodeSpan>;
    fn next(&mut self) -> Option<Self::Item> {
        let (kind_res, span) = self.iter.next()?;
        let span = CodeSpan {
            src_id: self.src_id,
            line: self.line,
            start: span.start,
            end: span.end,
        };

        if let Ok(kind) = kind_res {
            let text = &self.text[span.start..span.end];
            self.line += text.chars().filter(|c| *c == '\n').count();

            Some(Ok(Token { kind, span }))
        } else {
            Some(Err(span))
        }
    }
}

impl<'s> Lexer<'s> {
    pub fn new(src_id: usize, srcs: &'s SourceSequence) -> Self {
        let text = srcs.sources[src_id].text();
        let iter = TokenKind::lexer(text).spanned();
        let line = 0;
        Self {
            src_id,
            text,
            iter,
            line,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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
            Self::And => f.write_str("&&"),
            Self::Or => f.write_str("||"),
            Self::Not => f.write_str("!"),
        }
    }
}
