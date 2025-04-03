#include "ksc/lexer.h"
#include <cassert>
#include <cstring>
#include <iostream>

int main() {
    const char *str = "def fib(x) if x < 3 then 1 else fib(x-1) + fib(x-2)";
    int len = strlen(str);
    const KSCSource *ks_source =
        newKSCSource(KSC_SRC_STRING, str, len, nullptr, 0);

    if (ks_source == nullptr) {
        std::cerr << "Failed to create source" << std::endl;

        int err = getKSCSourceError();
        switch (err) {
        case KSC_SRC_ERR_EMPTY:
            std::cerr << "Source is empty" << std::endl;
            break;
        case KSC_SRC_ERR_UTF8:
            std::cerr << "Source is not valid UTF-8" << std::endl;
            break;
        case KSC_SRC_ERR_OK:
            std::cerr << "Unknown error" << std::endl;
            break;
        default:
            std::cerr << "Unknown error" << std::endl;
            break;
        }
        return 1;
    }

    const char *str_inner = getKSCSourceText(ks_source);

    if (str_inner == nullptr) {
        std::cerr << "Failed to get source text" << std::endl;
        return 1;
    }

    assert(std::strcmp(str_inner, str) == 0);
    freeKSCSource(ks_source);

    std::cout << "All tests passed" << std::endl;
    return 0;
}