# ksproject

为 _kaleidoscope_ 语言设计和实现的编译器课程实践项目。

使用 _Rust_ 和 _C++_ 作为主要开发语言：

- 前端使用 _logos_ 库和标准 _Rust_ 构建
- 后端使用 _LLVM_ 生成目标代码

---

## 目录结构

- kslang 前端实现
  - 词法解析器:
    - `kslang/src/compiler/lexer.rs`
    - `kslang/src/compiler/clexer.rs` （对外接口）
  - 语法解析器：
    - `kslang/src/compiler/ast.rs` (AST 定义)
    - `kslang/src/compiler/parser.rs` （Token 流解析函数组）

- kslangc 编译器 CLI 实现
  - lex 子命令 (词法分析)
    - `kslangc/src/cli/lex.rs`
  - ast 子命令 (语法分析)
    - `kslangc/src/cli/ast.rs`

- include 编译器前端对 C/C++ 语言程序接口
  - 自动导出接口
    - `include/ksc/_libkslang_autogen.h`
  - lexer 接口
    - `include/ksc/lexer.h` （C 接口）
    - `include/kslexer` （C++ 包装）
