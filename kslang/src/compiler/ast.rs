use super::lexer::{CodeSpan, Operator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ExprKind {
    Ident,
    Lit(f64),
    Parented(Box<Expr>),
    Block(Vec<Stmt>),

    /// foo(x, y, z)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        /// foo(x, y, z)
        ///    ^^^^^^^^^
        args_span: CodeSpan,
    },

    /// -x | !y
    UnOp {
        op: Operator,
        /// -x
        /// ^
        op_span: CodeSpan,
        arg: Box<Expr>,
    },

    /// x + y
    BinOp {
        op: Operator,
        /// x + y
        ///   ^
        op_span: CodeSpan,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    If {
        if_then_exprs: Vec<IfThenExpr>,
        /// if cond then then_branch else if cond2 then else_branch2 ...
        /// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        if_then_span: CodeSpan,
        else_branch: Option<Box<ElseExpr>>,
    },
    // For {
    //     loop_var: Box<Expr>,
    //     loop_range: Box<Expr>,
    //     /// for i in 0..10 { loop_body }
    //     /// ^^^^^^^^^^^^^^
    //     head_span: CodeSpan,
    //     loop_body: Box<Expr>,
    // },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IfThenExpr {
    pub cond: Expr,
    pub then: Expr,
    /// ... if cond then then_branch ...
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^
    pub span: CodeSpan,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElseExpr {
    pub expr: Expr,
    /// ... else else_branch
    ///     ^^^^^^^^^^^^^^^^
    pub span: CodeSpan,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: CodeSpan,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StmtKind {
    Assign {
        left: Expr,
        right: Expr,
        /// x = y
        ///   ^
        assign_span: CodeSpan,
    },

    Break,
    Continue,

    Def {
        ident: Expr,
        args: Vec<Expr>,
        /// def add(a, b) a + b
        ///        ^^^^^^
        args_span: CodeSpan,
        body: Expr,
        /// def add(a, b) a + b
        ///               ^^^^^
        body_span: CodeSpan,
    },

    Expr(Expr),

    /// extern add(a, b);
    Extern {
        ident: Expr,
        args: Vec<Expr>,
        /// extern add(a, b);
        ///           ^^^^^^
        args_span: CodeSpan,
    },

    Return(Expr),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: CodeSpan,
}
