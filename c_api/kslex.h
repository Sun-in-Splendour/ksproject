#ifndef KSLANG_H
#define KSLANG_H
#include "_libkslang_autogen.h"

extern "C" {

#define KS_TOKEN_NEWLINE 0
#define KS_TOKEN_KEYWORD 1
#define KS_TOKEN_IDENT 2
#define KS_TOKEN_NUMBER 3
#define KS_TOKEN_OPERATOR 4
#define KS_TOKEN_PUNCTUATOR 5

#define KS_KEYWORD_DEF 0
#define KS_KEYWORD_ELSE 1
#define KS_KEYWORD_EXTERN 2
#define KS_KEYWORD_FOR 3
#define KS_KEYWORD_IF 4
#define KS_KEYWORD_THEN 5

#define KS_OPERATOR_ASSIGN 0
#define KS_OPERATOR_EQ 1
#define KS_OPERATOR_NE 2
#define KS_OPERATOR_GT 3
#define KS_OPERATOR_GE 4
#define KS_OPERATOR_LT 5
#define KS_OPERATOR_LE 6
#define KS_OPERATOR_ADD 7
#define KS_OPERATOR_SUB 8
#define KS_OPERATOR_MUL 9
#define KS_OPERATOR_DIV 10
#define KS_OPERATOR_MOD 11

#define KS_PUNCTUATOR_OPEN_PAREN 0
#define KS_PUNCTUATOR_CLOSE_PAREN 1
#define KS_PUNCTUATOR_SEMICOLON 2

typedef uintptr_t KSIndexType;
typedef uint8_t const *KSStrType;

inline void ksc_lex_free(const KSCLexToken *tokens, const KSStrType bytes) {
    free((void *)tokens);
    free((void *)bytes);
}

inline int ksc_lex(
    // INPUT
    KSStrType src, KSIndexType src_len,
    // OUTPUT
    KSStrType *bytes, KSIndexType *bytes_len, const KSCLexToken **tokens,
    KSIndexType *tokens_len) {
    return __ksc_lex_lex(src, src_len, bytes, bytes_len, tokens, tokens_len);
}

inline KSIndexType ksc_token_get_kind(const KSCLexToken *token) {
    return token->kind;
}

inline double ksc_token_get_number(const KSCLexToken *token) {
    return token->value.double_;
}

inline KSIndexType ksc_token_get_index(const KSCLexToken *token) {
    return token->value.index;
}

} // extern "C"

#endif // KSLANG_H