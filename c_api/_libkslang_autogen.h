#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

union KSCLexTokenValue {
  uintptr_t index;
  double double_;
};

struct KSCLexToken {
  uint8_t kind;
  KSCLexTokenValue value;
  uintptr_t start;
  uintptr_t end;
};

extern "C" {

/// # Safety
/// - `src` 必须是有效的 UTF-8 字符串， 长度为 `src_len`
/// - `bytes`, `tokens` 将会被覆盖，调用者必须保证其内存安全
/// - `bytes_len`, `tokens_len` 必须是有效的指针
///
/// # 注意
/// - `bytes`, `tokens` 必须由调用者释放内存
int32_t __ksc_lex_lex(const uint8_t *src,
                      uintptr_t src_len,
                      const uint8_t **bytes,
                      uintptr_t *bytes_len,
                      const KSCLexToken **tokens,
                      uintptr_t *tokens_len);

}  // extern "C"
