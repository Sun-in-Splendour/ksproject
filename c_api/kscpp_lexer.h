#ifndef KSCPP_LEXER_H
#define KSCPP_LEXER_H

#include "ksc_lexer.h"
#include <iostream>
#include <string>

class KSLexerSource {
  public:
    KSLexerSource(KSCSourceKind kind, const std::string &str);
    ~KSLexerSource() { ks_c_lexer_source_free(source); }

  private:
    KSCSourceKind kind;
    KSCSource const *source;
};

class KSToken {
  public:
    KSToken(TokenKind kind, uintptr_t line, uintptr_t start, uintptr_t end)
        : kind(kind), line(line), start(start), end(end) {}

    inline bool is_keyword() const { return ks_c_token_is_keyword(kind); }
    inline bool is_operator() const { return ks_c_token_is_operator(kind); }
    inline bool is_punctuation() const {
        return ks_c_token_is_punctuation(kind);
    }

    inline std::ostream &operator<<(std::ostream &out) {
        out << "(" << ks_c_token_kind_name(kind);
        if (is_keyword()) {
            out << ", " << ks_c_keyword_as_str(kind);
        } else if (is_operator()) {
            out << ", " << ks_c_operator_as_str(kind);
        } else if (is_punctuation()) {
            out << ", " << ks_c_punctuation_as_str(kind);
        } else if (kind == KS_TOKEN_IDENT) {
            out << ", ";
        } else if (kind == KS_TOKEN_NUMBER) {
            out << ", ";
        }
        out << ")";
        return out;
    }

  private:
    TokenKind kind;
    uintptr_t line;
    uintptr_t start;
    uintptr_t end;
};

#endif // KSCPP_LEXER_H