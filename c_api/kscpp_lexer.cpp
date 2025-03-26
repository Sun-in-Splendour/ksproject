#include "kscpp_lexer.h"
#include "ksc_lexer.h"
#include <fstream>

KSLexerSource::KSLexerSource(KSCSourceKind kind, const std::string &source) {
    switch (kind) {
    case KSCSourceKind::Stdin:
    case KSCSourceKind::String:
        this->source = ks_c_lexer_source_new(kind, source.c_str(), nullptr);
        break;
    case KSCSourceKind::File:
        std::ifstream infile;
        infile.open(source);
        if (!infile.is_open()) {
            throw std::runtime_error("无法打开文件: " + source);
        }
        std::string text;
        std::string line;
        while (std::getline(infile, line)) {
            text += line + "\n";
        }
        infile.close();

        this->source =
            ks_c_lexer_source_new(kind, text.c_str(), source.c_str());

        break;
    }
}
