pub mod lexer;

mod ast;
mod clexer;
mod parser;

pub use parser::parse_stmt as parse_ast;

pub mod cextern {
    pub use super::clexer::*;
}
