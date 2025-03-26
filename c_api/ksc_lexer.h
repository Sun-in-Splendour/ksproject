#ifndef KS_C_LEXER_H
#define KS_C_LEXER_H

#ifndef KS_C_API_H
#include "_libkslang_autogen.h"
#endif // KS_C_API_H

#define KS_TOKEN_WHITESPACE 0
#define KS_TOKEN_COMMENT 1
#define KS_TOKEN_IDENT 2
#define KS_TOKEN_NUMBER 3

#define KS_TOKENS_KEYWORD KS_TOKEN_DEF

#define KS_TOKEN_DEF 4
#define KS_TOKEN_ELSE 5
#define KS_TOKEN_EXTERN 6
#define KS_TOKEN_FOR 7
#define KS_TOKEN_IF 8
#define KS_TOKEN_THEN 9

#define KS_TOKENS_OPERATOR KS_TOKEN_ASSIGN

#define KS_TOKEN_ASSIGN 10
#define KS_TOKEN_EQ 11
#define KS_TOKEN_NE 12
#define KS_TOKEN_GT 13
#define KS_TOKEN_GE 14
#define KS_TOKEN_LT 15
#define KS_TOKEN_LE 16
#define KS_TOKEN_ADD 17
#define KS_TOKEN_SUB 18
#define KS_TOKEN_MUL 19
#define KS_TOKEN_DIV 20
#define KS_TOKEN_MOD 21
#define KS_TOKEN_AND 22
#define KS_TOKEN_OR 23
#define KS_TOKEN_NOT 24

#define KS_TOKENS_PUNCTUATION KS_TOKEN_OPEN_PAREN

#define KS_TOKEN_OPEN_PAREN 25
#define KS_TOKEN_CLOSE_PAREN 26
#define KS_TOKEN_SEMICOLON 27

#define KS_TOKENS_LAST KS_TOKEN_SEMICOLON

typedef intptr_t TokenKind;

inline int ks_c_token_is_keyword(TokenKind kind) {
    return kind >= KS_TOKENS_KEYWORD && kind < KS_TOKENS_OPERATOR;
}

inline int ks_c_token_is_operator(TokenKind kind) {
    return kind >= KS_TOKENS_OPERATOR && kind < KS_TOKENS_PUNCTUATION;
}

inline int ks_c_token_is_punctuation(TokenKind kind) {
    return kind >= KS_TOKENS_PUNCTUATION;
}

inline int ks_c_token_kind_is_valid(TokenKind kind) {
    return kind >= 0 && kind <= KS_TOKENS_LAST;
}

inline const char *ks_c_token_kind_name(TokenKind kind) {
    switch (kind) {
    case KS_TOKEN_WHITESPACE:
        return "Whitespace";
    case KS_TOKEN_COMMENT:
        return "Comment";
    case KS_TOKEN_IDENT:
        return "Ident";
    case KS_TOKEN_NUMBER:
        return "Number";

    case KS_TOKEN_DEF:
    case KS_TOKEN_ELSE:
    case KS_TOKEN_EXTERN:
    case KS_TOKEN_FOR:
    case KS_TOKEN_IF:
    case KS_TOKEN_THEN:
        return "Keyword";

    case KS_TOKEN_ASSIGN:
    case KS_TOKEN_EQ:
    case KS_TOKEN_NE:
    case KS_TOKEN_GT:
    case KS_TOKEN_GE:
    case KS_TOKEN_LT:
    case KS_TOKEN_LE:
    case KS_TOKEN_ADD:
    case KS_TOKEN_SUB:
    case KS_TOKEN_MUL:
    case KS_TOKEN_DIV:
    case KS_TOKEN_MOD:
    case KS_TOKEN_AND:
    case KS_TOKEN_OR:
    case KS_TOKEN_NOT:
        return "Operator";

    case KS_TOKEN_OPEN_PAREN:
    case KS_TOKEN_CLOSE_PAREN:
    case KS_TOKEN_SEMICOLON:
        return "Punctuation";
    default:
        return "Unknown";
    }
}

inline const char *ks_c_keyword_as_str(TokenKind kind) {
    switch (kind) {
    case KS_TOKEN_DEF:
        return "def";
    case KS_TOKEN_ELSE:
        return "else";
    case KS_TOKEN_EXTERN:
        return "extern";
    case KS_TOKEN_FOR:
        return "for";
    case KS_TOKEN_IF:
        return "if";
    case KS_TOKEN_THEN:
        return "then";
    default:
        return "Unknown";
    }
}

inline const char *ks_c_operator_as_str(TokenKind kind) {
    switch (kind) {
    case KS_TOKEN_ASSIGN:
        return "=";
    case KS_TOKEN_EQ:
        return "==";
    case KS_TOKEN_NE:
        return "!=";
    case KS_TOKEN_GT:
        return ">";
    case KS_TOKEN_GE:
        return ">=";
    case KS_TOKEN_LT:
        return "<";
    case KS_TOKEN_LE:
        return "<=";
    case KS_TOKEN_ADD:
        return "+";
    case KS_TOKEN_SUB:
        return "-";
    case KS_TOKEN_MUL:
        return "*";
    case KS_TOKEN_DIV:
        return "/";
    case KS_TOKEN_MOD:
        return "%";
    case KS_TOKEN_AND:
        return "&&";
    case KS_TOKEN_OR:
        return "||";
    case KS_TOKEN_NOT:
        return "!";
    default:
        return "Unknown";
    }
}

inline const char *ks_c_punctuation_as_str(TokenKind kind) {
    switch (kind) {
    case KS_TOKEN_OPEN_PAREN:
        return "(";
    case KS_TOKEN_CLOSE_PAREN:
        return ")";
    case KS_TOKEN_SEMICOLON:
        return ";";
    default:
        return "Unknown";
    }
}

#endif // KS_C_LEXER_H