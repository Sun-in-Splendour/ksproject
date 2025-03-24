cmake_minimum_required(VERSION 3.10.0)
project(tests VERSION 0.1.0 LANGUAGES C CXX)

add_executable(ctests tests/test_line_lex.cpp)
target_include_directories(ctests PUBLIC c_api)

if(NOT DEFINED KSLANG_TARGET_DIR)
    message(FATAL_ERROR
        "请提供kslang库文件所在目录，一般为 `<ksproject_dir>/target/<debug|release>`")
endif()

target_link_libraries(ctests PUBLIC ${KSLANG_TARGET_DIR}/libkslang.a)

