#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Source
struct KSCSource {

};

using KSCSourceKind = uintptr_t;

using KSCSourceErr = uintptr_t;

constexpr static const KSCSourceKind KSC_SRC_STDIN = 0;

constexpr static const KSCSourceKind KSC_SRC_STRING = 1;

constexpr static const KSCSourceKind KSC_SRC_FILE = 2;

constexpr static const KSCSourceErr KSC_SRC_ERR_OK = 0;

constexpr static const KSCSourceErr KSC_SRC_ERR_EMPTY = 1;

constexpr static const KSCSourceErr KSC_SRC_ERR_UTF8 = 2;

extern "C" {

/// # Safety
const KSCSource *newKSCSource(KSCSourceKind src_type,
                              const char *src_data,
                              uintptr_t src_len,
                              const char *src_path,
                              uintptr_t src_path_len);

/// # Safety
KSCSourceErr getKSCSourceError();

/// # Safety
const char *getKSCSourceText(const KSCSource *src);

/// # Safety
void freeKSCSource(const KSCSource *src);

}  // extern "C"
