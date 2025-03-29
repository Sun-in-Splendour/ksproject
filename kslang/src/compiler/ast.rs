use super::lexer::{DebugSpan, Operator};

type P<T> = Box<T>;

#[derive(Debug, Clone)]
pub enum ExprKind {
    Ident,
    Lit(f64),
    Parented(P<Expr>),
    Block(Vec<Stmt>),

    /// foo(x, y, z)
    Call {
        callee: P<Expr>,
        args: Vec<Expr>,
        /// foo(x, y, z)
        ///    ^^^^^^^^^
        args_span: DebugSpan,
    },

    /// -x | !y
    UnOp {
        op: Operator,
        /// -x
        /// ^
        op_span: DebugSpan,
        arg: P<Expr>,
    },

    /// x + y
    BinOp {
        op: Operator,
        /// x + y
        ///   ^
        op_span: DebugSpan,
        left: P<Expr>,
        right: P<Expr>,
    },

    If {
        if_then_exprs: Vec<IfThenExpr>,
        /// if cond then then_branch else if cond2 then else_branch2 ...
        /// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        if_then_span: DebugSpan,
        else_branch: Option<P<ElseExpr>>,
    },

    For {
        loop_var: P<Expr>,
        loop_range: P<Expr>,
        /// for i in 0..10 { loop_body }
        /// ^^^^^^^^^^^^^^
        head_span: DebugSpan,
        loop_body: P<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct IfThenExpr {
    pub cond: Expr,
    pub then: Expr,
    /// ... if cond then then_branch ...
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^
    pub span: DebugSpan,
}

impl IfThenExpr {
    fn is_evalable(&self) -> bool {
        self.then.is_evalable()
    }
}

#[derive(Debug, Clone)]
pub struct ElseExpr {
    pub expr: Expr,
    /// ... else else_branch
    ///     ^^^^^^^^^^^^^^^^
    pub span: DebugSpan,
}

impl ElseExpr {
    fn is_evalable(&self) -> bool {
        self.expr.is_evalable()
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: DebugSpan,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Assign {
        left: Expr,
        right: Expr,
        /// x = y
        ///   ^
        assign_span: DebugSpan,
    },

    Break,
    Continue,

    Def {
        ident: Expr,
        args: Vec<Expr>,
        /// def add(a, b) a + b
        ///        ^^^^^^
        body: Expr,
        /// def add(a, b) a + b
        ///               ^^^^^
        body_span: DebugSpan,
    },

    Expr(Expr),

    /// extern add(a, b);
    Extern {
        ident: Expr,
        args: Vec<Expr>,
        /// extern add(a, b);
        ///           ^^^^^^
        args_span: DebugSpan,
    },

    Return(Expr),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: DebugSpan,
}

impl Expr {
    pub fn is_evalable(&self) -> bool {
        match &self.kind {
            ExprKind::Lit(_)
            | ExprKind::Ident
            | ExprKind::Call { .. }
            | ExprKind::UnOp { .. }
            | ExprKind::BinOp { .. } => true,
            ExprKind::For { .. } => false,
            ExprKind::Parented(e) => e.is_evalable(),

            ExprKind::Block(stmts) => {
                if let Some(stmt) = stmts.last() {
                    stmt.is_evalable()
                } else {
                    false
                }
            }

            ExprKind::If {
                if_then_exprs,
                else_branch,
                ..
            } => {
                if let Some(else_) = else_branch {
                    if_then_exprs.iter().all(|e| e.is_evalable()) && else_.is_evalable()
                } else {
                    false
                }
            }
        }
    }
}

impl Stmt {
    pub fn is_evalable(&self) -> bool {
        todo!()
    }
}
