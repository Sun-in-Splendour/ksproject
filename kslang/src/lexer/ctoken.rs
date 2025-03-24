use super::token::{Token, TokenValue, Tokenizer};

enum TokenKind {
    NewLine,
    Keyword,
    Ident,
    Number,
    Operator,
    Punctuation,
}

#[repr(C)]
pub union KSCLexTokenValue {
    pub index: usize,
    pub double_: f64,
}

#[repr(C)]
pub struct KSCLexToken {
    pub kind: u8,
    pub value: KSCLexTokenValue,
    pub start: usize,
    pub end: usize,
}

macro_rules! tv {
    (i $k:ident, $v:expr) => {
        (TokenKind::$k as u8, KSCLexTokenValue { index: $v })
    };
    (d $k:ident, $v:expr) => {
        (TokenKind::$k as u8, KSCLexTokenValue { double_: $v })
    };
}

impl Token {
    pub fn to_c_token(self, bytes: &mut Vec<u8>) -> KSCLexToken {
        let (kind, value) = match self.val {
            TokenValue::NewLine => tv!(i NewLine, 0),
            TokenValue::Keyword(k) => tv!(i Keyword, k as usize),
            TokenValue::Ident(i) => {
                let index = bytes.len();
                bytes.extend_from_slice(i.as_bytes());
                bytes.push(0);
                tv!(i Ident, index)
            }
            TokenValue::Number(n) => tv!(d Number, n),
            TokenValue::Operator(o) => tv!(i Operator, o as usize),
            TokenValue::Punctuation(p) => tv!(i Punctuation, p as usize),
        };

        KSCLexToken {
            kind,
            value,
            start: self.loc.start,
            end: self.loc.end,
        }
    }
}

/// # Safety
/// - `src` 必须是有效的 UTF-8 字符串， 长度为 `src_len`
/// - `bytes`, `tokens` 将会被覆盖，调用者必须保证其内存安全
/// - `bytes_len`, `tokens_len` 必须是有效的指针
///
/// # 注意
/// - `bytes`, `tokens` 必须由调用者释放内存
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ksc_lex_lex(
    // INPUT
    src: *const u8,
    src_len: usize,
    // OUTPUT
    bytes: *mut *const u8,
    bytes_len: *mut usize,
    tokens: *mut *const KSCLexToken,
    tokens_len: *mut usize,
) -> i32 {
    let src = unsafe { std::slice::from_raw_parts(src, src_len) };

    let mut ctokens = Vec::new();
    let mut strbytes = Vec::new();
    let mut err_cnt = 0;
    if let Ok(s) = std::str::from_utf8(src) {
        let iter = Tokenizer::new(s);

        for token in iter {
            if let Ok(tk) = token {
                ctokens.push(tk.to_c_token(&mut strbytes));
            } else {
                err_cnt -= 1;
            }
        }
    }

    let strbytes_len = strbytes.len();
    let ctokens_len = ctokens.len();

    let ctokens_ptr = ctokens.as_ptr();
    let bytes_ptr = strbytes.as_ptr();

    // 手动管理内存, 必须由调用者释放内存
    std::mem::forget(ctokens);
    std::mem::forget(strbytes);

    unsafe {
        *bytes_len = strbytes_len;
        *tokens_len = ctokens_len;
        *tokens = ctokens_ptr;
        *bytes = bytes_ptr;
    }

    err_cnt
}
