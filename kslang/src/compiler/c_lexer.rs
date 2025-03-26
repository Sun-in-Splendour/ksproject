use super::lexer::{DebugSpan, Lexer, Source, Token};
use std::{
    ffi::{CStr, c_char},
    path::Path,
};

/// 词法单元 （Wraps `Token`）
#[repr(C)]
pub struct KSCToken;

/// 词法分析结果 （Wraps `Result<Token, DebugSpan>`）
#[repr(C)]
pub struct KSCTokenResult;

/// 源代码 （Wraps `Source`）
#[repr(C)]
pub struct KSCSource;

/// 词法分析器 （Wraps `Lexer`）
#[repr(C)]
pub struct KSCLexer;

#[repr(C)]
pub enum KSCSourceKind {
    /// 从标准输入读取源代码
    Stdin,
    /// 从字符串中读取源代码
    String,
    /// 从文件中读取源代码
    File,
}

/// # Safety
///
/// - `source_text` `source_path` 必须是有效的 UTF-8 字符串
/// -  请使用 `ks_c_source_free` 来释放 `KSCSource` 指针
///
/// # 返回值
///
///     成功：返回一个 `KSCSource` 指针
///     失败：返回 `NULL`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_lexer_source_new(
    source_kind: KSCSourceKind,
    source_text: *const c_char,
    source_path: *const c_char,
) -> *const KSCSource {
    if source_text.is_null() {
        return std::ptr::null();
    }

    let text = unsafe { CStr::from_ptr(source_text) };
    let source = if let Ok(text) = text.to_str() {
        match source_kind {
            KSCSourceKind::Stdin => Source::Stdin(text.to_string()),
            KSCSourceKind::String => Source::String(text.to_string()),
            KSCSourceKind::File => {
                if source_path.is_null() {
                    return std::ptr::null();
                }

                let path = unsafe { CStr::from_ptr(source_path) };
                if let Ok(path) = path.to_str() {
                    Source::File {
                        path: Path::new(path).to_path_buf(),
                        contents: text.to_string(),
                    }
                } else {
                    return std::ptr::null();
                }
            }
        }
    } else {
        return std::ptr::null();
    };

    let ptr = Box::into_raw(Box::new(source));
    ptr as *const KSCSource
}

/// # Safety
///
/// - `source` 必须是有效的指针
/// - `source` 取 NULL 值是安全的
/// - 请使用 `ks_c_lexer_free` 来释放 `KSCLexer` 指针
///
/// # 返回值
///
///     成功：返回一个 `KSCToken` 指针
///     失败：返回 `NULL`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_lexer_new(source: *const KSCSource) -> *mut KSCLexer {
    if source.is_null() {
        return std::ptr::null_mut();
    }
    let source = unsafe { &*(source as *const Source) };
    let lexer = Lexer::new(source);
    let ptr = Box::into_raw(Box::new(lexer));
    ptr as *mut KSCLexer
}

/// # Safety
///
/// - `source` 取 NULL 值是安全的
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_lexer_source_free(source: *const KSCSource) {
    if !source.is_null() {
        unsafe { _ = Box::from_raw(source as *mut Source) };
    }
}

/// # Safety
///
/// - `source` 必须是有效的指针
/// - `source` 取 NULL 值是安全的
pub unsafe extern "C" fn ks_c_lexer_source_text(source: *const KSCSource) -> *const c_char {
    if source.is_null() {
        std::ptr::null()
    } else {
        let source = unsafe { &*(source as *const Source) };
        let text = source.text();
        text as *const str as *const c_char
    }
}

/// # Safety
///
/// - `lexer` 取 NULL 值是安全的
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_lexer_free(lexer: *mut KSCLexer) {
    if !lexer.is_null() {
        unsafe { _ = Box::from_raw(lexer as *mut Lexer) };
    }
}

/// # Safety
///
/// - `lexer` 必须是有效的指针
///
/// # 返回值
///
///     成功：返回一个 `KSCTokenResult` 指针
///     失败：返回 `NULL`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_lexer_next(lexer: *mut KSCLexer) -> *const KSCTokenResult {
    if lexer.is_null() {
        std::ptr::null()
    } else {
        let lexer = unsafe { &mut *(lexer as *mut Lexer) };
        let token = lexer.next();
        if let Some(token) = token {
            Box::into_raw(Box::new(token)) as *const KSCTokenResult
        } else {
            std::ptr::null()
        }
    }
}

/// # Safety
///
/// - `result` 必须是有效的指针
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_token_result_free(result: *const KSCTokenResult) {
    if !result.is_null() {
        unsafe { _ = Box::from_raw(result as *mut Result<Token<'_>, DebugSpan>) };
    }
}

/// # Safety
///
/// - `result` 必须是有效的指针
///
/// # 返回值
///
/// `result` 为 `NULL` 时返回 `false`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_token_result_is_ok(result: *const KSCTokenResult) -> bool {
    if result.is_null() {
        false
    } else {
        let result = unsafe { &*(result as *const Result<Token<'_>, DebugSpan>) };
        result.is_ok()
    }
}

/// # Safety
///
/// - `result` 必须是有效的指针
/// - `result` 必须是 `Ok`
///
/// # 返回值
///
/// 返回一个 `KSCToken` 指针, 如果 `result` 为 `NULL` 或 `Err` 返回 `NULL`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_get_token(result: *const KSCTokenResult) -> *const KSCToken {
    if result.is_null() {
        std::ptr::null()
    } else {
        let result = unsafe { &*(result as *const Result<Token, DebugSpan>) };
        if let Ok(token) = result {
            token as *const Token<'_> as *const KSCToken
        } else {
            std::ptr::null()
        }
    }
}

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 类型， 如果 `token` 为 `NULL` 返回 `-1`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_token_get_kind(token: *const KSCToken) -> isize {
    if token.is_null() {
        -1
    } else {
        let token = unsafe { &*(token as *const Token) };
        token.kind as isize
    }
}

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 行数， 如果 `token` 为 `NULL` 返回 usize::MAX
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_token_get_line(token: *const KSCToken) -> usize {
    if token.is_null() {
        usize::MAX
    } else {
        let token = unsafe { &*(token as *const Token) };
        token.span.line
    }
}

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 起始位置， 如果 `token` 为 `NULL` 返回 usize::MAX
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_token_get_span_start(token: *const KSCToken) -> usize {
    if token.is_null() {
        usize::MAX
    } else {
        let token = unsafe { &*(token as *const Token) };
        token.span.start
    }
}

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 结束位置， 如果 `token` 为 `NULL` 返回 usize::MAX
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ks_c_token_get_span_end(token: *const KSCToken) -> usize {
    if token.is_null() {
        usize::MAX
    } else {
        let token = unsafe { &*(token as *const Token) };
        token.span.end
    }
}
