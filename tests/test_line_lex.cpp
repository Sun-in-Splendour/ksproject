#include <cstring>
#include <iostream>
#include <string>

#include "kslex.h"

void print_keyword(KSIndexType i) {
    switch (i) {
    case KS_KEYWORD_DEF:
        std::cout << "`def`";
        break;
    case KS_KEYWORD_ELSE:
        std::cout << "`else`";
        break;
    case KS_KEYWORD_EXTERN:
        std::cout << "`extern`";
        break;
    case KS_KEYWORD_FOR:
        std::cout << "`for`";
        break;
    case KS_KEYWORD_IF:
        std::cout << "`if`";
        break;
    case KS_KEYWORD_THEN:
        std::cout << "`then`";
        break;
    }
}

void print_operator(KSIndexType i) {
    switch (i) {
    case KS_OPERATOR_ASSIGN:
        std::cout << "`=`";
        break;
    case KS_OPERATOR_EQ:
        std::cout << "`==`";
        break;
    case KS_OPERATOR_NE:
        std::cout << "`!=`";
        break;
    case KS_OPERATOR_GT:
        std::cout << "`>`";
        break;
    case KS_OPERATOR_GE:
        std::cout << "`>=`";
        break;
    case KS_OPERATOR_LT:
        std::cout << "`<`";
        break;
    case KS_OPERATOR_LE:
        std::cout << "`<=`";
        break;
    case KS_OPERATOR_ADD:
        std::cout << "`+`";
        break;
    case KS_OPERATOR_SUB:
        std::cout << "`-`";
        break;
    case KS_OPERATOR_MUL:
        std::cout << "`*`";
        break;
    case KS_OPERATOR_DIV:
        std::cout << "`/`";
        break;
    case KS_OPERATOR_MOD:
        std::cout << "`%`";
        break;
    }
}

void print_punctuator(KSIndexType i) {
    switch (i) {
    case KS_PUNCTUATOR_OPEN_PAREN:
        std::cout << "`(`";
        break;
    case KS_PUNCTUATOR_CLOSE_PAREN:
        std::cout << "`)`";
        break;
    case KS_PUNCTUATOR_SEMICOLON:
        std::cout << "`;`";
        break;
    }
}

void print_ident(KSStrType bytes, KSIndexType index, KSIndexType len) {
    if (index >= len) {
        std::cout << "<Error>";
    }

    std::string s;
    size_t p = index;
    while (bytes[p] != '\0' && p < len) {
        s.push_back((char)bytes[p]);
        p++;
    }
    std::cout << "`" << s << "`";
}

int main() {
    std::string s;
    std::cout << "line >>> ";
    std::getline(std::cin, s);

    const char *src = s.c_str();
    KSIndexType src_len = (KSIndexType)strlen(src);

    KSStrType bytes = nullptr;
    KSIndexType bytes_len = 0;
    const KSCLexToken *tokens = nullptr;
    KSIndexType tokens_len = 0;

    int i = ksc_lex((KSStrType)src, src_len, &bytes, &bytes_len, &tokens,
                    &tokens_len);

    if (tokens == nullptr) {
        std::cerr << "Error: 无法解析" << std::endl;
        return 1;
    }

    for (KSIndexType j = 0; j < tokens_len; j++) {
        switch (ksc_token_get_kind(&tokens[j])) {
        case KS_TOKEN_NEWLINE:
            std::cout << "(NEWLINE)" << std::endl;
            break;
        case KS_TOKEN_KEYWORD:
            std::cout << "(KEYWORD, ";
            print_keyword(ksc_token_get_index(&tokens[j]));
            std::cout << ")" << std::endl;
            break;
        case KS_TOKEN_IDENT:
            std::cout << "(IDENT, ";
            print_ident(bytes, ksc_token_get_index(&tokens[j]), bytes_len);
            std::cout << ")" << std::endl;
            break;
        case KS_TOKEN_NUMBER:
            std::cout << "(NUMBER, " << ksc_token_get_number(&tokens[j]) << ")"
                      << std::endl;
            break;
        case KS_TOKEN_OPERATOR:
            std::cout << "(OPERATOR, ";
            print_operator(ksc_token_get_index(&tokens[j]));
            std::cout << ")" << std::endl;
            break;
        case KS_TOKEN_PUNCTUATOR:
            std::cout << "(PUNCTUATOR, ";
            print_punctuator(ksc_token_get_index(&tokens[j]));
            std::cout << ")" << std::endl;
            break;
        }
    }
    std::cout << std::endl;

    ksc_lex_free(tokens, bytes);

    if (i < 0) {
        std::cerr << "Error: " << i << "个错误" << std::endl;
        return 1;
    }
    return 0;
}
