use logos::{Logos, SpannedIter};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Range};

#[derive(Logos, Debug, Clone, Copy)]
#[logos(skip r"[ \t\r\v\f]+")]
pub enum LexUnit {
    #[token("\n")]
    NewLine,
    #[regex(r"#.*")]
    Comment,
    #[regex(r"[\p{L}_][\p{L}\p{N}_]*")]
    Ident,
    #[regex(r"(\d[\d_]*(\._*\d[\d_]*)?(e_*[+-]?_*\d[\d_]*)?)")]
    Number,

    // Keywords
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

    // Operators
    #[token("=")]
    Assign,

    //     Comparison Operators
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

    //     Arithmetic Operators
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

    // Punctuation
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token(";")]
    Semicolon,
}

impl LexUnit {
    #[doc(hidden)]
    pub fn _lexer(source: &str) -> SpannedIter<LexUnit> {
        Self::lexer(source).spanned()
    }

    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            LexUnit::Def
                | LexUnit::Else
                | LexUnit::Extern
                | LexUnit::For
                | LexUnit::If
                | LexUnit::Then
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            LexUnit::Assign
                | LexUnit::Eq
                | LexUnit::Ne
                | LexUnit::Gt
                | LexUnit::Ge
                | LexUnit::Lt
                | LexUnit::Le
                | LexUnit::Add
                | LexUnit::Sub
                | LexUnit::Mul
                | LexUnit::Div
                | LexUnit::Mod
        )
    }

    pub fn is_punctuation(&self) -> bool {
        matches!(
            self,
            LexUnit::OpenParen | LexUnit::CloseParen | LexUnit::Semicolon
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum Keyword {
    Def,
    Else,
    Extern,
    For,
    If,
    Then,
}

impl Keyword {
    fn as_str(&self) -> &'static str {
        match self {
            Keyword::Def => "def",
            Keyword::Else => "else",
            Keyword::Extern => "extern",
            Keyword::For => "for",
            Keyword::If => "if",
            Keyword::Then => "then",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum Operator {
    Assign,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Operator {
    fn as_str(&self) -> &'static str {
        match self {
            Operator::Assign => "=",
            Operator::Eq => "==",
            Operator::Ne => "!=",
            Operator::Gt => ">",
            Operator::Ge => ">=",
            Operator::Lt => "<",
            Operator::Le => "<=",
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Mod => "%",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum Punctuation {
    OpenParen,
    CloseParen,
    Semicolon,
}

impl Punctuation {
    fn as_str(&self) -> &'static str {
        match self {
            Punctuation::OpenParen => "(",
            Punctuation::CloseParen => ")",
            Punctuation::Semicolon => ";",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenValue {
    NewLine,
    Keyword(Keyword),
    Ident(String),
    Number(f64),
    Operator(Operator),
    Punctuation(Punctuation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub val: TokenValue,
    pub loc: Range<usize>,
}

impl Token {
    pub fn as_binary_string(&self) -> String {
        match &self.val {
            TokenValue::NewLine => "(NewLine)".to_string(),
            TokenValue::Keyword(k) => format!("(Keyword, {})", k.as_str()),
            TokenValue::Ident(i) => format!("(Ident, {})", i),
            TokenValue::Number(n) => format!("(Number, {})", n),
            TokenValue::Operator(o) => format!("(Operator, {})", o.as_str()),
            TokenValue::Punctuation(p) => format!("(Punc, {})", p.as_str()),
        }
    }
}

pub struct Tokenizer<'s> {
    source: &'s str,
    lexer: SpannedIter<'s, LexUnit>,
}

impl<'s> Tokenizer<'s> {
    pub fn new(source: &'s str) -> Self {
        Tokenizer {
            source,
            lexer: LexUnit::lexer(source).spanned(),
        }
    }
}

#[derive(Debug)]
pub enum TokenizeError {
    UnmatchedUnit(Range<usize>),
    NumberParseError(Range<usize>),
}

impl TokenizeError {
    pub fn location(&self) -> &Range<usize> {
        match self {
            TokenizeError::UnmatchedUnit(loc) => loc,
            TokenizeError::NumberParseError(loc) => loc,
        }
    }
}

impl std::fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenizeError::UnmatchedUnit(loc) => {
                write!(f, "`{}..{}`处没有匹配的Token", loc.start, loc.end)
            }
            TokenizeError::NumberParseError(loc) => {
                write!(f, "`{}..{}`处的数字无法解析", loc.start, loc.end)
            }
        }
    }
}

impl std::error::Error for TokenizeError {}

macro_rules! t {
    (newline $l:expr) => {{
        let val = TokenValue::NewLine;
        Some(Ok(Token { val, loc: $l }))
    }};
    (str $self:expr, $l:expr) => {{
        let src = $self.source[$l.clone()].to_string();
        let val = TokenValue::Ident(src);
        Some(Ok(Token { val, loc: $l }))
    }};
    (num $self:expr, $l:expr) => {{
        let src = $self.source[$l.clone()].to_string();
        if let Ok(num) = src.parse() {
            let val = TokenValue::Number(num);
            Some(Ok(Token { val, loc: $l }))
        } else {
            Some(Err(TokenizeError::NumberParseError($l)))
        }
    }};
    (kw $k:ident, $l:expr) => {{
        let val = TokenValue::Keyword(Keyword::$k);
        Some(Ok(Token { val, loc: $l }))
    }};
    (op $o:ident, $l:expr) => {{
        let val = TokenValue::Operator(Operator::$o);
        Some(Ok(Token { val, loc: $l }))
    }};
    (punc $p:ident, $l:expr) => {{
        let val = TokenValue::Punctuation(Punctuation::$p);
        Some(Ok(Token { val, loc: $l }))
    }};
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (unit, loc) = self.lexer.next()?;

        if let Ok(unit) = unit {
            match unit {
                LexUnit::NewLine => t!(newline loc),
                LexUnit::Comment => self.next(),
                LexUnit::Ident => t!(str self, loc),
                LexUnit::Number => t!(num self, loc),

                // Keywords
                LexUnit::Def => t!(kw Def, loc),
                LexUnit::Else => t!(kw Else, loc),
                LexUnit::Extern => t!(kw Extern, loc),
                LexUnit::For => t!(kw For, loc),
                LexUnit::If => t!(kw If, loc),
                LexUnit::Then => t!(kw Then, loc),

                // Operators
                LexUnit::Assign => t!(op Assign, loc),

                //     Comparison Operators
                LexUnit::Eq => t!(op Eq, loc),
                LexUnit::Ne => t!(op Ne, loc),
                LexUnit::Gt => t!(op Gt, loc),
                LexUnit::Ge => t!(op Ge, loc),
                LexUnit::Lt => t!(op Lt, loc),
                LexUnit::Le => t!(op Le, loc),

                //     Arithmetic Operators
                LexUnit::Add => t!(op Add, loc),
                LexUnit::Sub => t!(op Sub, loc),
                LexUnit::Mul => t!(op Mul, loc),
                LexUnit::Div => t!(op Div, loc),
                LexUnit::Mod => t!(op Mod, loc),

                // Punctuation
                LexUnit::OpenParen => t!(punc OpenParen, loc),
                LexUnit::CloseParen => t!(punc CloseParen, loc),
                LexUnit::Semicolon => t!(punc Semicolon, loc),
                // _ => unreachable!(),
            }
        } else {
            Some(Err(TokenizeError::UnmatchedUnit(loc)))
        }
    }
}
