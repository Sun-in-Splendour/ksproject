# ksproject

为 _kaleidoscope_ 语言设计和实现的编译器课程实践项目。

使用 _Rust_ 和 _C++_ 作为主要开发语言：

- 前端使用 _logos_ 库和标准 _Rust_ 构建
- 后端使用 _LLVM_ 生成目标代码

====

#### kslang

前端实现。包括：

- 词法解析器:
  - `kslang/src/compiler/lexer.rs`
  - `kslang/src/compiler/c_lexer.rs` （对外接口）
- 语法解析器：
  - `kslang/src/compiler/ast.rs` (AST 定义)
  - `kslang/src/compiler/parser.rs` （Token 流解析函数组）


#### kslangc

编译器 CLI 实现。包括：

- lexer 子命令
  - `kslangc/src/cli/lexer.rs`


#### c_api

编译器前端对 C/C++ 语言程序接口。包括：

- 自动导出接口
  - `c_api/_libkslang_autogen.h`
- lexer 接口
  - `c_api/ksc_lexer.h` （C 接口）
  - `c_api/kscpp_lexer.h` （C++ 包装）
