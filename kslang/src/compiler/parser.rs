use super::{
    CodeSpan,
    ast::{ElseExpr, Expr, ExprKind, IfThenExpr, Stmt, StmtKind},
    lexer::{Operator, Token, TokenKind},
};

type Res<'s, T> = Result<(T, &'s [Token], CodeSpan), ParseError>;
type Ctx<'s> = (&'s str, &'s [Token], CodeSpan);

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEnd(CodeSpan),
    UnexpectedToken(Token),

    AssignRightExprError(Box<Self>),
    ConsecutiveAssignError(CodeSpan),
    ReturnExprError(Box<Self>),

    ArgsNonBegin(CodeSpan),
    ArgsNonEnd(CodeSpan),
    ArgsExprError(Box<Self>),
    ArgsCommaError(CodeSpan),

    ForIdentError(CodeSpan),
    ForInError(CodeSpan),
    ForIterError(Box<Self>),
    ForBodyError(Box<Self>),

    DefNameError(CodeSpan),
    DefArgsError(Box<Self>),
    DefBodyError(Box<Self>),

    ExternNameError(CodeSpan),
    ExternArgsError(Box<Self>),
    ExternEndError(CodeSpan),

    FloatError(Token),
    CondExprError(Box<Self>),
    ThenTokenError(Token),
    ThenExprError(Box<Self>),
}

macro_rules! next_or_ret {
    ($($e:expr),* $(,)?) => {
        $(match $e {
            Ok(t) => return Ok(t),
            Err(err) => match err {
                ParseError::UnexpectedToken(_) => {}
                _ => return Err(err),
            },
        })*
    };
}

fn parse_skips(ctx: Ctx) -> Res<&Token> {
    let (src, tokens, span) = ctx;

    let (token, rest) = tokens
        .split_first()
        .ok_or(ParseError::UnexpectedEnd(span))?;

    if !matches!(
        token.kind,
        TokenKind::Whitespace | TokenKind::Comment | TokenKind::UTF8BOM
    ) {
        Ok((token, rest, token.span))
    } else {
        parse_skips((src, rest, token.span))
    }
}

pub fn parse_ast(src: &str, tokens: &[Token]) -> Result<Vec<Stmt>, ParseError> {
    if tokens.is_empty() || src.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut stmts = Vec::new();
    let mut rest = tokens;
    let mut last_span = CodeSpan {
        line: 0,
        src_id: 0,
        start: 0,
        end: 0,
    };

    while parse_skips((src, rest, last_span)).is_ok() {
        match parse_stmt((src, rest, last_span)) {
            Ok((stmt, stmt_rest, span)) => {
                stmts.push(stmt);
                rest = stmt_rest;
                last_span = span;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(stmts)
}

fn parse_stmt(ctx: Ctx) -> Res<Stmt> {
    let (src, rest, span) = ctx;

    next_or_ret! {
        parse_empty(ctx),
        parse_assign(ctx),
        parse_return(ctx),
        parse_break_continue(ctx),
        parse_for_loop(ctx),
        parse_def(ctx),
        parse_extern(ctx),
    };

    match parse_expr((src, rest, span)) {
        Ok((expr, rest, last_span)) => {
            let span = expr.span;
            let kind = StmtKind::Expr(expr);
            let (_, rest, last_span) = parse_semi((src, rest, last_span))?;
            Ok((Stmt { kind, span }, rest, last_span))
        }
        Err(e) => Err(e),
    }
}

#[inline(always)]
fn parse_expr(ctx: Ctx) -> Res<Expr> {
    parse_range(ctx)
}

fn parse_semi(ctx: Ctx) -> Res<()> {
    let (_, rest, span) = ctx;
    if let Ok((semi, semi_rest, semi_span)) = parse_skips(ctx) {
        if matches!(semi.kind, TokenKind::Semicolon) {
            return Ok(((), semi_rest, semi_span));
        }
    }
    Ok(((), rest, span))
}

fn parse_assign(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;

    let (ident, ident_rest, _) = parse_skips(ctx)?;
    if !matches!(ident.kind, TokenKind::Ident) || ident_rest.is_empty() {
        return Err(ParseError::UnexpectedToken(*ident));
    }

    let left = Expr {
        kind: ExprKind::Ident,
        span: ident.span,
    };

    let (op, op_rest, _) = parse_skips((src, ident_rest, ident.span))?;
    if !matches!(op.kind, TokenKind::Assign) {
        return Err(ParseError::UnexpectedToken(*op));
    }

    let assign_span = op.span;

    let (right, rest, right_last_span) = parse_expr((src, op_rest, op.span))
        .map_err(|e| ParseError::AssignRightExprError(Box::new(e)))?;

    let span = ident.span.merge(right.span);
    let kind = StmtKind::Assign {
        left,
        right,
        assign_span,
    };

    if let Ok((a, _, _)) = parse_skips((src, rest, right_last_span)) {
        if matches!(a.kind, TokenKind::Assign) {
            let span = ident.span.merge(a.span);
            return Err(ParseError::ConsecutiveAssignError(span));
        }
    }

    let (_, rest, last_span) = parse_semi((src, rest, right_last_span))?;
    Ok((Stmt { kind, span }, rest, last_span))
}

fn parse_return(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;
    let (token, rest, _) = parse_skips(ctx)?;
    if !matches!(token.kind, TokenKind::Return) {
        return Err(ParseError::UnexpectedToken(*token));
    }

    let (expr, rest, expr_last_span) = parse_expr((src, rest, token.span))
        .map_err(|e| ParseError::ReturnExprError(Box::new(e)))?;
    let span = token.span.merge(expr.span);
    let kind = StmtKind::Return(expr);

    let (_, rest, last_span) = parse_semi((src, rest, expr_last_span))?;
    Ok((Stmt { kind, span }, rest, last_span))
}

fn parse_args(ctx: Ctx) -> Res<(Vec<Expr>, CodeSpan)> {
    let (src, _, _) = ctx;

    let (open_paren, rest, _) = parse_skips(ctx)?;
    if !matches!(open_paren.kind, TokenKind::OpenParen) {
        return Err(ParseError::ArgsNonBegin(open_paren.span));
    }

    let (mut args, mut rest, mut last_span) = (Vec::new(), rest, open_paren.span);
    let mut flag = false;
    while let Ok((arg, arg_rest, arg_last_span)) = parse_expr((src, rest, last_span)) {
        args.push(arg);
        rest = arg_rest;
        last_span = arg_last_span;

        let (c, c_rest, c_last_span) = parse_skips((src, rest, last_span))?;
        if !matches!(c.kind, TokenKind::Comma) {
            flag = true;
            break;
        }

        rest = c_rest;
        last_span = c_last_span;
    }

    let (close_paren, rest, _) = parse_skips((src, rest, last_span))?;
    if !matches!(close_paren.kind, TokenKind::CloseParen) {
        if flag {
            return Err(ParseError::ArgsCommaError(close_paren.span));
        } else {
            return Err(ParseError::ArgsNonEnd(close_paren.span));
        }
    }

    let span = open_paren.span.merge(close_paren.span);
    Ok(((args, span), rest, close_paren.span))
}

fn parse_extern(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;
    let (extern_token, rest, _) = parse_skips(ctx)?;
    if !matches!(extern_token.kind, TokenKind::Extern) {
        return Err(ParseError::UnexpectedToken(*extern_token));
    }

    let (ident, ident_rest, _) = parse_skips((src, rest, extern_token.span))?;
    if !matches!(ident.kind, TokenKind::Ident) {
        return Err(ParseError::ExternNameError(ident.span));
    }

    let ((args, args_span), args_rest, args_last_span) = parse_args((src, ident_rest, ident.span))
        .map_err(|e| ParseError::ExternArgsError(Box::new(e)))?;

    let (s_token, s_rest, _) = parse_skips((src, args_rest, args_last_span))?;
    if !matches!(s_token.kind, TokenKind::Semicolon) {
        return Err(ParseError::ExternEndError(s_token.span));
    }

    let span = extern_token.span.merge(s_token.span);
    let ident = Expr {
        kind: ExprKind::Ident,
        span: ident.span,
    };
    let kind = StmtKind::Extern {
        ident,
        args,
        args_span,
    };
    Ok((Stmt { kind, span }, s_rest, s_token.span))
}

fn parse_def(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;
    let (def, rest, _) = parse_skips(ctx)?;
    if !matches!(def.kind, TokenKind::Def) {
        return Err(ParseError::UnexpectedToken(*def));
    }

    let (ident, ident_rest, _) = parse_skips((src, rest, def.span))?;
    if !matches!(ident.kind, TokenKind::Ident) {
        return Err(ParseError::DefNameError(ident.span));
    }

    let ((args, args_span), args_rest, args_last_span) = parse_args((src, ident_rest, ident.span))
        .map_err(|e| ParseError::DefArgsError(Box::new(e)))?;

    let (body, body_rest, body_last_span) = parse_expr((src, args_rest, args_last_span))
        .map_err(|e| ParseError::DefBodyError(Box::new(e)))?;

    let span = def.span.merge(body.span);

    let ident = Expr {
        kind: ExprKind::Ident,
        span: ident.span,
    };
    let body_span = body.span;
    let kind = StmtKind::Def {
        ident,
        args,
        args_span,
        body,
        body_span,
    };

    let (_, rest, last_span) = parse_semi((src, body_rest, body_last_span))?;
    Ok((Stmt { kind, span }, rest, last_span))
}

fn parse_break_continue(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;

    let (token, rest, _) = parse_skips(ctx)?;
    if !matches!(token.kind, TokenKind::Break | TokenKind::Continue) {
        return Err(ParseError::UnexpectedToken(*token));
    }

    let span = token.span;
    let kind = match token.kind {
        TokenKind::Break => StmtKind::Break,
        TokenKind::Continue => StmtKind::Continue,
        _ => unreachable!(),
    };

    let (_, rest, last_span) = parse_semi((src, rest, span))?;
    Ok((Stmt { kind, span }, rest, last_span))
}

fn parse_for_loop(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;
    let (for_token, for_rest, _) = parse_skips(ctx)?;
    if !matches!(for_token.kind, TokenKind::For) {
        return Err(ParseError::UnexpectedToken(*for_token));
    }

    let (ident, ident_rest, _) = parse_skips((src, for_rest, for_token.span))?;
    if !matches!(ident.kind, TokenKind::Ident) {
        return Err(ParseError::ForIdentError(ident.span));
    }

    let (in_token, in_rest, _) = parse_skips((src, ident_rest, ident.span))?;
    if !matches!(in_token.kind, TokenKind::In) {
        return Err(ParseError::ForInError(in_token.span));
    }

    let (iter, iter_rest, iter_last_span) = parse_expr((src, in_rest, in_token.span))
        .map_err(|e| ParseError::ForIterError(Box::new(e)))?;
    let (body, rest, body_last_span) = parse_expr((src, iter_rest, iter_last_span))
        .map_err(|e| ParseError::ForBodyError(Box::new(e)))?;

    let span = for_token.span.merge(body.span);
    let ident = Expr {
        kind: ExprKind::Ident,
        span: ident.span,
    };
    let kind = StmtKind::For {
        loop_var: Box::new(ident),
        loop_iter: Box::new(iter),
        head_span: for_token.span.merge(iter_last_span),
        loop_body: Box::new(body),
    };

    let (_, rest, last_span) = parse_semi((src, rest, body_last_span))?;
    Ok((Stmt { kind, span }, rest, last_span))
}

fn parse_empty(ctx: Ctx) -> Res<Stmt> {
    let (semi, rest, span) = parse_skips(ctx)?;
    if !matches!(semi.kind, TokenKind::Semicolon) {
        return Err(ParseError::UnexpectedToken(*semi));
    }
    let stmt = Stmt {
        kind: StmtKind::Empty,
        span,
    };
    Ok((stmt, rest, span))
}

macro_rules! parse_binop {
    ($n:ident, $i:ident, $tk:ident => $bk:block) => {
        fn $n(ctx: Ctx) -> Res<Expr> {
            let (src, _, _) = ctx;

            let (mut left, mut rest, mut last_span) = $i(ctx)?;
            while let Ok((op_token, op_rest, op_span)) = parse_skips((src, rest, last_span)) {
                let $tk = op_token.kind;
                let op = if let Some(op) = $bk {
                    op
                } else {
                    break;
                };

                let (right, right_rest, right_last_span) = $i((src, op_rest, op_span))?;

                let span = left.span.merge(right.span);
                left = Expr {
                    kind: ExprKind::BinOp {
                        op,
                        op_span,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    span,
                };
                rest = right_rest;
                last_span = right_last_span;
            }
            Ok((left, rest, last_span))
        }
    };
}

parse_binop! {
    parse_range, parse_and_or, kind => {
        match kind {
            TokenKind::Range => Some(Operator::Range),
            _ => None,
        }
    }
}

parse_binop! {
    parse_and_or, parse_comparision, kind => {
        match kind {
            TokenKind::And => Some(Operator::And),
            TokenKind::Or => Some(Operator::Or),
            _ => None,
        }
    }
}

parse_binop! {
    parse_comparision, parse_add_sub, kind => {
        match kind {
            TokenKind::Eq => Some(Operator::Eq),
            TokenKind::Ne => Some(Operator::Ne),
            TokenKind::Lt => Some(Operator::Lt),
            TokenKind::Le => Some(Operator::Le),
            TokenKind::Gt => Some(Operator::Gt),
            TokenKind::Ge => Some(Operator::Ge),
            _ => None,
        }
    }
}

parse_binop! {
    parse_add_sub, parse_mul_div, kind => {
        match kind {
            TokenKind::Add => Some(Operator::Add),
            TokenKind::Sub => Some(Operator::Sub),
            _ => None,
        }
    }
}

parse_binop! {
    parse_mul_div, parse_unary, kind => {
        match kind {
            TokenKind::Mul => Some(Operator::Mul),
            TokenKind::Div => Some(Operator::Div),
            _ => None,
        }
    }
}

fn parse_unary(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;

    let (op_token, rest, _) = parse_skips(ctx)?;
    if !matches!(op_token.kind, TokenKind::Not | TokenKind::Sub) {
        return parse_call(ctx);
    }

    let op_span = op_token.span;
    let (expr, rest, last_span) = parse_call((src, rest, op_span))?;
    let span = op_token.span.merge(expr.span);
    let op = match op_token.kind {
        TokenKind::Not => Operator::Not,
        TokenKind::Sub => Operator::Sub,
        _ => unreachable!(),
    };

    let kind = ExprKind::UnOp {
        op,
        op_span,
        arg: Box::new(expr),
    };
    Ok((Expr { kind, span }, rest, last_span))
}

fn parse_call(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;
    let (ident, ident_rest, _) = parse_skips(ctx)?;
    if !matches!(ident.kind, TokenKind::Ident) {
        return parse_primary(ctx);
    }

    match parse_args((src, ident_rest, ident.span)) {
        Ok(((args, args_span), args_rest, args_last_span)) => {
            let span = ident.span.merge(args_span);
            let ident = Expr {
                kind: ExprKind::Ident,
                span: ident.span,
            };
            let kind = ExprKind::Call {
                callee: Box::new(ident),
                args,
                args_span,
            };
            Ok((Expr { kind, span }, args_rest, args_last_span))
        }
        Err(ParseError::ArgsNonBegin(_)) => parse_primary(ctx),
        Err(e) => Err(e),
    }
}

fn parse_primary(ctx: Ctx) -> Res<Expr> {
    let (_, _, span) = ctx;

    next_or_ret! {
        parse_block(ctx),
        parse_parented(ctx),
        parse_if_expr(ctx),
        parse_lit(ctx),
        parse_ident(ctx),
        parse_ellipsis(ctx),
    };

    Err(ParseError::UnexpectedEnd(span))
}

fn parse_block(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;

    let (open_brace, rest, _) = parse_skips(ctx)?;
    if !matches!(open_brace.kind, TokenKind::OpenBrace) {
        return Err(ParseError::UnexpectedToken(*open_brace));
    }

    let (mut stmts, mut rest, mut last_span) = (Vec::new(), rest, open_brace.span);
    while let Ok((stmt, stmt_rest, stmt_last_span)) = parse_stmt((src, rest, last_span)) {
        stmts.push(stmt);
        rest = stmt_rest;
        last_span = stmt_last_span;
    }

    let (close_brace, rest, _) = parse_skips((src, rest, last_span))?;
    if !matches!(close_brace.kind, TokenKind::CloseBrace) {
        return Err(ParseError::UnexpectedToken(*close_brace));
    }

    let span = open_brace.span.merge(close_brace.span);
    let kind = ExprKind::Block(stmts);
    Ok((Expr { kind, span }, rest, close_brace.span))
}

fn parse_if_then<'s>(if_token: &'s Token, ctx: Ctx<'s>) -> Res<'s, IfThenExpr> {
    let (src, _, _) = ctx;
    let (cond, cond_rest, cond_last_span) =
        parse_expr(ctx).map_err(|e| ParseError::CondExprError(Box::new(e)))?;

    let (then_token, then_rest, _) = parse_skips((src, cond_rest, cond_last_span))?;
    if !matches!(then_token.kind, TokenKind::Then) {
        return Err(ParseError::ThenTokenError(*then_token));
    }

    let (then, expr_rest, expr_last_span) = parse_expr((src, then_rest, then_token.span))
        .map_err(|e| ParseError::ThenExprError(Box::new(e)))?;
    let span = if_token.span.merge(then.span);
    Ok((IfThenExpr { cond, then, span }, expr_rest, expr_last_span))
}

fn parse_if_expr(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;
    let (if_token, if_rest, _) = parse_skips(ctx)?;
    if !matches!(if_token.kind, TokenKind::If) {
        return Err(ParseError::UnexpectedToken(*if_token));
    }

    let (if_then, rest, if_then_last_span) =
        parse_if_then(if_token, (src, if_rest, if_token.span))?;

    let mut if_then_exprs = vec![if_then];
    let mut else_branch = None;
    let mut rest = rest;
    let mut last_span = if_then_last_span;

    while let Ok((else_token, else_rest, else_last_span)) = parse_skips((src, rest, last_span)) {
        // no else
        if !matches!(else_token.kind, TokenKind::Else) {
            break;
        }

        let (else_if_token, else_if_rest, _) = parse_skips((src, else_rest, else_last_span))?;

        // no else if but has else
        if !matches!(else_if_token.kind, TokenKind::If) {
            let (expr, expr_rest, expr_last_span) = parse_expr((src, else_rest, else_last_span))?;
            let span = else_last_span.merge(expr.span);
            rest = expr_rest;
            last_span = expr_last_span;
            else_branch = Some(Box::new(ElseExpr { expr, span }));
            break;
        }

        let (else_if_then, else_if_then_rest, else_if_then_last_span) =
            parse_if_then(else_if_token, (src, else_if_rest, else_if_token.span))?;

        if_then_exprs.push(else_if_then);
        rest = else_if_then_rest;
        last_span = else_if_then_last_span;
    }

    let mut if_then_span = if_then_exprs[0].span;
    if if_then_exprs.len() > 1 {
        if_then_span = if_then_span.merge(if_then_exprs.last().unwrap().span);
    }

    let kind = ExprKind::If {
        if_then_exprs,
        if_then_span,
        else_branch,
    };

    let span = if_token.span.merge(last_span);
    Ok((Expr { kind, span }, rest, last_span))
}

fn parse_parented(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;

    let (open_paren, rest, _) = parse_skips(ctx)?;
    if !matches!(open_paren.kind, TokenKind::OpenParen) {
        return Err(ParseError::UnexpectedToken(*open_paren));
    }

    let (expr, rest, last_span) = parse_expr((src, rest, open_paren.span))?;

    let (close_paren, rest, _) = parse_skips((src, rest, last_span))?;
    if !matches!(close_paren.kind, TokenKind::CloseParen) {
        Err(ParseError::UnexpectedToken(*close_paren))
    } else {
        let span = open_paren.span.merge(close_paren.span);
        let kind = ExprKind::Parented(Box::new(expr));
        Ok((Expr { kind, span }, rest, close_paren.span))
    }
}

fn parse_lit(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;
    let (token, rest, _) = parse_skips(ctx)?;
    if !matches!(token.kind, TokenKind::Number) {
        return Err(ParseError::UnexpectedToken(*token));
    }

    let span = token.span;
    if let Ok(value) = src[span.start..span.end].parse() {
        let kind = ExprKind::Lit(value);
        Ok((Expr { kind, span }, rest, span))
    } else {
        Err(ParseError::FloatError(*token))
    }
}

fn parse_ident(ctx: Ctx) -> Res<Expr> {
    let (token, rest, _) = parse_skips(ctx)?;
    if !matches!(token.kind, TokenKind::Ident) {
        Err(ParseError::UnexpectedToken(*token))
    } else {
        let span = token.span;
        let kind = ExprKind::Ident;
        Ok((Expr { kind, span }, rest, span))
    }
}

fn parse_ellipsis(ctx: Ctx) -> Res<Expr> {
    let (token, rest, _) = parse_skips(ctx)?;
    if !matches!(token.kind, TokenKind::Ellipsis) {
        Err(ParseError::UnexpectedToken(*token))
    } else {
        let span = token.span;
        let kind = ExprKind::Ellipsis;
        Ok((Expr { kind, span }, rest, span))
    }
}
