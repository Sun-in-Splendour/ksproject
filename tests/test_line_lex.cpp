#include "ksc_lexer.h"
#include <iostream>
#include <string>

int main() {
    std::cout << "line >>> ";
    std::string input;
    std::getline(std::cin, input);

    KSCSource const *source =
        ks_c_lexer_source_new(KSCSourceKind::Stdin, input.c_str(), nullptr);

    if (source == nullptr) {
        std::cout << "无法创建源";
        return -1;
    }

    KSCLexer *lexer = ks_c_lexer_new(source);

    if (lexer == nullptr) {
        std::cout << "无法创建词法分析器";
        ks_c_lexer_source_free(source);
        return -1;
    }

    while (true) {
        KSCTokenResult const *res = ks_c_lexer_next(lexer);
        if (res == nullptr) {
            break;
        } else if (!ks_c_token_result_is_ok(res)) {
            ks_c_token_result_free(res);
            std::cout << "词法分析错误" << std::endl;
            break;
        }

        KSCToken const *token = ks_c_get_token(res);
        if (token == nullptr) {
            std::cout << "无法获取词法单元" << std::endl;
            ks_c_token_result_free(res);
            break;
        }

        TokenKind kind = ks_c_token_get_kind(token);
        uintptr_t start = ks_c_token_get_span_start(token);
        uintptr_t end = ks_c_token_get_span_end(token);

        std::cout << "(" << ks_c_token_kind_name(kind);
        if (ks_c_token_is_keyword(kind)) {
            std::cout << ", " << ks_c_keyword_as_str(kind);
        } else if (ks_c_token_is_operator(kind)) {
            std::cout << ", " << ks_c_operator_as_str(kind);
        } else if (ks_c_token_is_punctuation(kind)) {
            std::cout << ", " << ks_c_punctuation_as_str(kind);
        } else if (kind != KS_TOKEN_WHITESPACE && kind != KS_TOKEN_COMMENT) {
            std::cout << ", `";
            for (uintptr_t i = start; i < end; i++) {
                std::cout << input[i];
            }
            std::cout << "`";
        }
        std::cout << ") " << std::endl;

        ks_c_token_result_free(res);
    }

    ks_c_lexer_source_free(source);
    ks_c_lexer_free(lexer);
    return 0;
}
