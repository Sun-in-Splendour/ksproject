use super::{
    ast::{ElseExpr, Expr, ExprKind, IfThenExpr, Stmt, StmtKind},
    lexer::{CodeSpan, Operator, Token, TokenKind},
};

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEnd(CodeSpan),
    UnexpectedToken(Token),
    FloatError(Token),
}

type Res<'s, T> = Result<(T, &'s [Token], CodeSpan), ParseError>;
type Ctx<'s> = (&'s str, &'s [Token], CodeSpan);

fn parse_skips(ctx: Ctx) -> Res<&Token> {
    let (src, tokens, span) = ctx;

    let (token, rest) = tokens
        .split_first()
        .ok_or(ParseError::UnexpectedEnd(span))?;

    if !matches!(token.kind, TokenKind::Whitespace | TokenKind::Comment) {
        Ok((token, rest, token.span))
    } else {
        parse_skips((src, rest, token.span))
    }
}

pub fn parse_stmt(ctx: Ctx) -> Res<Stmt> {
    let (_, rest, span) = ctx;

    if let Ok(tup) = parse_assign(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_return(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_break_continue(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_def(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_extern(ctx) {
        Ok(tup)
    } else if let Ok((expr, rest, last_span)) = parse_expr(ctx) {
        let span = expr.span;
        let kind = StmtKind::Expr(expr);
        Ok((Stmt { kind, span }, rest, last_span))
    } else if !rest.is_empty() {
        Err(ParseError::UnexpectedToken(rest[0]))
    } else {
        Err(ParseError::UnexpectedEnd(span))
    }
}

#[inline(always)]
fn parse_expr(ctx: Ctx) -> Res<Expr> {
    parse_and_or(ctx)
}

fn parse_assign(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;

    let (ident, ident_rest, _) = parse_skips(ctx)?;
    if !matches!(ident.kind, TokenKind::Ident) {
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
    let (right, rest, right_last_span) = parse_expr((src, op_rest, op.span))?;

    let span = ident.span.merge(right.span);
    let kind = StmtKind::Assign {
        left,
        right,
        assign_span,
    };
    Ok((Stmt { kind, span }, rest, right_last_span))
}

fn parse_return(ctx: Ctx) -> Res<Stmt> {
    let (src, _, _) = ctx;
    let (token, rest, _) = parse_skips(ctx)?;
    if !matches!(token.kind, TokenKind::Return) {
        return Err(ParseError::UnexpectedToken(*token));
    }

    let (expr, rest, expr_last_span) = parse_expr((src, rest, token.span))?;
    let span = token.span.merge(expr.span);
    let kind = StmtKind::Return(expr);
    Ok((Stmt { kind, span }, rest, expr_last_span))
}

fn parse_args(ctx: Ctx) -> Res<(Vec<Expr>, CodeSpan)> {
    let (src, _, _) = ctx;

    let (open_paren, rest, _) = parse_skips(ctx)?;
    if !matches!(open_paren.kind, TokenKind::OpenParen) {
        return Err(ParseError::UnexpectedToken(*open_paren));
    }

    let (mut args, mut rest, mut last_span) = (Vec::new(), rest, open_paren.span);
    while let Ok((arg, arg_rest, arg_last_span)) = parse_expr((src, rest, last_span)) {
        args.push(arg);
        rest = arg_rest;
        last_span = arg_last_span;

        let (c, c_rest, c_last_span) = parse_skips((src, rest, last_span))?;
        if !matches!(c.kind, TokenKind::Comma) {
            break;
        }

        rest = c_rest;
        last_span = c_last_span;
    }

    let (close_paren, rest, _) = parse_skips((src, rest, last_span))?;
    if !matches!(close_paren.kind, TokenKind::CloseParen) {
        return Err(ParseError::UnexpectedToken(*close_paren));
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
        return Err(ParseError::UnexpectedToken(*ident));
    }

    let ((args, args_span), args_rest, args_last_span) = parse_args((src, ident_rest, ident.span))?;

    let (s_token, s_rest, _) = parse_skips((src, args_rest, args_last_span))?;
    if !matches!(s_token.kind, TokenKind::Semicolon) {
        return Err(ParseError::UnexpectedToken(*s_token));
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
        return Err(ParseError::UnexpectedToken(*ident));
    }

    let ((args, args_span), args_rest, args_last_span) = parse_args((src, ident_rest, ident.span))?;
    let (body, body_rest, body_last_span) = parse_expr((src, args_rest, args_last_span))?;

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
    Ok((Stmt { kind, span }, body_rest, body_last_span))
}

fn parse_break_continue(ctx: Ctx) -> Res<Stmt> {
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
    Ok((Stmt { kind, span }, rest, span))
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
        return parse_primary(ctx);
    }

    let op_span = op_token.span;
    let (expr, rest, last_span) = parse_primary((src, rest, op_span))?;
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

fn parse_primary(ctx: Ctx) -> Res<Expr> {
    let (_, _, span) = ctx;

    if let Ok(tup) = parse_if_expr(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_call(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_block(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_parented(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_lit(ctx) {
        Ok(tup)
    } else if let Ok(tup) = parse_ident(ctx) {
        Ok(tup)
    } else {
        Err(ParseError::UnexpectedEnd(span))
    }
}

fn parse_call(ctx: Ctx) -> Res<Expr> {
    let (src, _, _) = ctx;
    let (ident, ident_rest, _) = parse_skips(ctx)?;
    if !matches!(ident.kind, TokenKind::Ident) {
        return Err(ParseError::UnexpectedToken(*ident));
    }

    let ((args, args_span), args_rest, args_last_span) = parse_args((src, ident_rest, ident.span))?;
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
    let (cond, cond_rest, cond_last_span) = parse_expr(ctx)?;

    let (then_token, then_rest, _) = parse_skips((src, cond_rest, cond_last_span))?;
    if !matches!(then_token.kind, TokenKind::Then) {
        return Err(ParseError::UnexpectedToken(*then_token));
    }

    let (then, expr_rest, expr_last_span) = parse_expr((src, then_rest, then_token.span))?;
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
