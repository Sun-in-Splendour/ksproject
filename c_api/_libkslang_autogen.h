#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class KSCSourceKind {
  /// 从标准输入读取源代码
  Stdin,
  /// 从字符串中读取源代码
  String,
  /// 从文件中读取源代码
  File,
};

/// 源代码 （Wraps `Source`）
struct KSCSource {

};

/// 词法分析器 （Wraps `Lexer`）
struct KSCLexer {

};

/// 词法分析结果 （Wraps `Result<Token, DebugSpan>`）
struct KSCTokenResult {

};

/// 词法单元 （Wraps `Token`）
struct KSCToken {

};

extern "C" {

/// # Safety
///
/// - `source_text` `source_path` 必须是有效的 UTF-8 字符串
/// -  请使用 `ks_c_source_free` 来释放 `KSCSource` 指针
///
/// # 返回值
///
///     成功：返回一个 `KSCSource` 指针
///     失败：返回 `NULL`
const KSCSource *ks_c_lexer_source_new(KSCSourceKind source_kind,
                                       const char *source_text,
                                       const char *source_path);

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
KSCLexer *ks_c_lexer_new(const KSCSource *source);

/// # Safety
///
/// - `source` 取 NULL 值是安全的
void ks_c_lexer_source_free(const KSCSource *source);

/// # Safety
///
/// - `lexer` 取 NULL 值是安全的
void ks_c_lexer_free(KSCLexer *lexer);

/// # Safety
///
/// - `lexer` 必须是有效的指针
///
/// # 返回值
///
///     成功：返回一个 `KSCTokenResult` 指针
///     失败：返回 `NULL`
const KSCTokenResult *ks_c_lexer_next(KSCLexer *lexer);

/// # Safety
///
/// - `result` 必须是有效的指针
///
void ks_c_token_result_free(const KSCTokenResult *result);

/// # Safety
///
/// - `result` 必须是有效的指针
///
/// # 返回值
///
/// `result` 为 `NULL` 时返回 `false`
bool ks_c_token_result_is_ok(const KSCTokenResult *result);

/// # Safety
///
/// - `result` 必须是有效的指针
/// - `result` 必须是 `Ok`
///
/// # 返回值
///
/// 返回一个 `KSCToken` 指针, 如果 `result` 为 `NULL` 或 `Err` 返回 `NULL`
const KSCToken *ks_c_get_token(const KSCTokenResult *result);

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 类型， 如果 `token` 为 `NULL` 返回 `-1`
intptr_t ks_c_token_get_kind(const KSCToken *token);

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 行数， 如果 `token` 为 `NULL` 返回 usize::MAX
uintptr_t ks_c_token_get_line(const KSCToken *token);

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 起始位置， 如果 `token` 为 `NULL` 返回 usize::MAX
uintptr_t ks_c_token_get_span_start(const KSCToken *token);

/// # Safety
///
/// - `token` 必须是有效的指针
///
/// # 返回值
///
/// 返回 Token 结束位置， 如果 `token` 为 `NULL` 返回 usize::MAX
uintptr_t ks_c_token_get_span_end(const KSCToken *token);

}  // extern "C"
