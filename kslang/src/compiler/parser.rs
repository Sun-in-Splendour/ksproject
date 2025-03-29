use super::{
    ast::{ElseExpr, Expr, ExprKind, IfThenExpr, Stmt, StmtKind},
    lexer::{Operator, Token, TokenKind},
};

#[derive(Debug, Clone)]
pub enum ParseError<'s> {
    UnexpectedEnd,
    UnexpectedToken(Token<'s>),
    FloatParseError(Token<'s>),
}

pub type ParseResult<'s, T> = Result<(T, &'s [Token<'s>]), ParseError<'s>>;

pub fn parse_stmt<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Stmt> {
    if let Ok(tup) = parse_break_continue(tokens) {
        Ok(tup)
    } else if let Ok(tup) = parse_return(tokens) {
        Ok(tup)
    } else if let Ok(tup) = parse_assign(tokens) {
        Ok(tup)
    } else if let Ok(tup) = parse_expr(tokens) {
        let span = tup.0.span;
        let kind = StmtKind::Expr(tup.0);
        Ok((Stmt { kind, span }, tup.1))
    } else if !tokens.is_empty() {
        Err(ParseError::UnexpectedToken(tokens[0].clone()))
    } else {
        Err(ParseError::UnexpectedEnd)
    }
}

fn parse_assign<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Stmt> {
    let (ident_token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(ident_token.kind, TokenKind::Ident) {
        return Err(ParseError::UnexpectedToken(ident_token.clone()));
    }

    let left = Expr {
        kind: ExprKind::Ident,
        span: ident_token.span,
    };

    let (eq_token, rest) = skip_ws_split_first(rest)?;
    if !matches!(eq_token.kind, TokenKind::Assign) {
        return Err(ParseError::UnexpectedToken(eq_token.clone()));
    }
    let assign_span = eq_token.span;
    let (right, rest) = parse_expr(rest)?;

    let span = ident_token.span.merge(right.span);
    let kind = StmtKind::Assign {
        left,
        right,
        assign_span,
    };
    Ok((Stmt { kind, span }, rest))
}

fn parse_break_continue<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Stmt> {
    let (token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(token.kind, TokenKind::Break | TokenKind::Continue) {
        return Err(ParseError::UnexpectedToken(token.clone()));
    }

    let span = token.span;
    let kind = match token.kind {
        TokenKind::Break => StmtKind::Break,
        TokenKind::Continue => StmtKind::Continue,
        _ => unreachable!(),
    };
    Ok((Stmt { kind, span }, rest))
}

fn parse_return<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Stmt> {
    let (token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(token.kind, TokenKind::Return) {
        return Err(ParseError::UnexpectedToken(token.clone()));
    }

    let (expr, rest) = parse_expr(rest)?;
    let span = token.span.merge(expr.span);
    let kind = StmtKind::Return(expr);
    Ok((Stmt { kind, span }, rest))
}

fn parse_if_then<'s>(if_token: &Token<'s>, tokens: &'s [Token<'s>]) -> ParseResult<'s, IfThenExpr> {
    let (cond, rest) = parse_expr(tokens)?;
    let (then_token, rest) = skip_ws_split_first(rest)?;
    if !matches!(then_token.kind, TokenKind::Then) {
        return Err(ParseError::UnexpectedToken(then_token.clone()));
    }
    let (then, rest) = parse_expr(rest)?;
    let span = if_token.span.merge(then.span);
    Ok((IfThenExpr { cond, then, span }, rest))
}

fn parse_if<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (if_token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(if_token.kind, TokenKind::If) {
        return Err(ParseError::UnexpectedToken(if_token.clone()));
    }

    let if_token_span = if_token.span;
    let (if_then, rest) = parse_if_then(if_token, rest)?;

    let mut last_span = if_then.span;
    let mut if_then_exprs = vec![if_then];
    let mut else_branch = None;
    let mut rest = rest;

    while let Ok((else_token, new_rest)) = skip_ws_split_first(rest) {
        // no else
        if !matches!(else_token.kind, TokenKind::Else) {
            break;
        }

        let else_rest = new_rest;
        let (else_if_token, new_rest) = skip_ws_split_first(new_rest)?;

        // no else if but has else
        if !matches!(else_if_token.kind, TokenKind::If) {
            // println!("rest = {:#?}", new_rest);
            let (expr, new_rest) = parse_expr(else_rest)?;
            let span = else_token.span.merge(expr.span);

            last_span = span;
            rest = new_rest;
            else_branch = Some(Box::new(ElseExpr { expr, span }));
            break;
        }

        let (else_if_then, new_rest) = parse_if_then(else_if_token, new_rest)?;

        last_span = else_if_then.span;
        if_then_exprs.push(else_if_then);
        rest = new_rest;
    }

    let if_then_span = if_then_exprs[0].span;
    let if_then_span = if_then_span.merge(if_then_exprs.last().unwrap().span);

    let kind = ExprKind::If {
        if_then_exprs,
        if_then_span,
        else_branch,
    };
    let span = if_token_span.merge(last_span);
    Ok((Expr { kind, span }, rest))
}

#[inline(always)]
fn parse_expr<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    parse_bool_op(tokens)
}

fn skip_ws_split_first<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, &'s Token<'s>> {
    let (token, rest) = tokens.split_first().ok_or(ParseError::UnexpectedEnd)?;
    if !matches!(token.kind, TokenKind::Whitespace) {
        Ok((token, rest))
    } else {
        skip_ws_split_first(rest)
    }
}

fn parse_lit<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(token.kind, TokenKind::Number) {
        return Err(ParseError::UnexpectedToken(token.clone()));
    }

    let (text, span) = (token.text, token.span);
    if let Ok(value) = text.parse() {
        let kind = ExprKind::Lit(value);
        Ok((Expr { kind, span }, rest))
    } else {
        Err(ParseError::FloatParseError(token.clone()))
    }
}

fn parse_ident<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(token.kind, TokenKind::Ident) {
        Err(ParseError::UnexpectedToken(token.clone()))
    } else {
        let span = token.span;
        let kind = ExprKind::Ident;
        Ok((Expr { kind, span }, rest))
    }
}

fn parse_parented<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (open_paren, rest) = skip_ws_split_first(tokens)?;
    if !matches!(open_paren.kind, TokenKind::OpenParen) {
        return Err(ParseError::UnexpectedToken(open_paren.clone()));
    }

    let (expr, rest) = parse_expr(rest)?;

    let (close_paren, rest) = skip_ws_split_first(rest)?;
    if !matches!(close_paren.kind, TokenKind::CloseParen) {
        Err(ParseError::UnexpectedToken(close_paren.clone()))
    } else {
        let span = open_paren.span.merge(close_paren.span);
        let kind = ExprKind::Parented(Box::new(expr));
        Ok((Expr { kind, span }, rest))
    }
}

fn parse_block<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (open_brace, rest) = skip_ws_split_first(tokens)?;
    if !matches!(open_brace.kind, TokenKind::OpenBrace) {
        return Err(ParseError::UnexpectedToken(open_brace.clone()));
    }

    let mut stmts = Vec::new();
    let mut rest = rest;
    while let Ok((stmt, new_rest)) = parse_stmt(rest) {
        stmts.push(stmt);
        rest = new_rest;
    }

    let (close_brace, rest) = skip_ws_split_first(rest)?;
    if !matches!(close_brace.kind, TokenKind::CloseBrace) {
        Err(ParseError::UnexpectedToken(close_brace.clone()))
    } else {
        let span = open_brace.span.merge(close_brace.span);
        let kind = ExprKind::Block(stmts);
        Ok((Expr { kind, span }, rest))
    }
}

fn parse_primary<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    if let Ok(tup) = parse_block(tokens) {
        Ok(tup)
    } else if let Ok(tup) = parse_if(tokens) {
        Ok(tup)
    } else if let Ok(tup) = parse_parented(tokens) {
        Ok(tup)
    } else if let Ok(tup) = parse_lit(tokens) {
        Ok(tup)
    } else {
        parse_ident(tokens)
    }
}

fn parse_unary<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (op_token, rest) = skip_ws_split_first(tokens)?;
    if !matches!(op_token.kind, TokenKind::Not | TokenKind::Sub) {
        return parse_primary(tokens);
    }

    let op_span = op_token.span;
    let (expr, rest) = parse_primary(rest)?;
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
    Ok((Expr { kind, span }, rest))
}

fn parse_mul_div_mod<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (mut left, mut rest) = parse_unary(tokens)?;

    while let Ok((op_token, new_rest)) = skip_ws_split_first(rest) {
        let op = match op_token.kind {
            TokenKind::Mul => Operator::Mul,
            TokenKind::Div => Operator::Div,
            TokenKind::Mod => Operator::Mod,
            _ => break,
        };

        let op_span = op_token.span;
        let (right, next_rest) = parse_primary(new_rest)?;

        let span = left.span.merge(right.span).merge(right.span);
        left = Expr {
            kind: ExprKind::BinOp {
                op,
                op_span,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        };
        rest = next_rest;
    }

    Ok((left, rest))
}

fn parse_add_sub<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (mut left, mut rest) = parse_mul_div_mod(tokens)?;

    while let Ok((op_token, new_rest)) = skip_ws_split_first(rest) {
        let op = match op_token.kind {
            TokenKind::Add => Operator::Add,
            TokenKind::Sub => Operator::Sub,
            _ => break,
        };

        let op_span = op_token.span;
        let (right, next_rest) = parse_mul_div_mod(new_rest)?;

        let span = left.span.merge(right.span).merge(right.span);
        left = Expr {
            kind: ExprKind::BinOp {
                op,
                op_span,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        };
        rest = next_rest;
    }

    Ok((left, rest))
}

fn parse_comparision<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (mut left, mut rest) = parse_add_sub(tokens)?;

    while let Ok((op_token, new_rest)) = skip_ws_split_first(rest) {
        let op = match op_token.kind {
            TokenKind::Eq => Operator::Eq,
            TokenKind::Ne => Operator::Ne,
            TokenKind::Lt => Operator::Lt,
            TokenKind::Le => Operator::Le,
            TokenKind::Gt => Operator::Gt,
            TokenKind::Ge => Operator::Ge,
            _ => break,
        };

        let op_span = op_token.span;
        let (right, next_rest) = parse_add_sub(new_rest)?;

        let span = left.span.merge(right.span).merge(right.span);
        left = Expr {
            kind: ExprKind::BinOp {
                op,
                op_span,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        };
        rest = next_rest;
    }

    Ok((left, rest))
}

fn parse_bool_op<'s>(tokens: &'s [Token<'s>]) -> ParseResult<'s, Expr> {
    let (mut left, mut rest) = parse_comparision(tokens)?;

    while let Ok((op_token, new_rest)) = skip_ws_split_first(rest) {
        let op = match op_token.kind {
            TokenKind::And => Operator::And,
            TokenKind::Or => Operator::Or,
            _ => break,
        };

        let op_span = op_token.span;
        let (right, next_rest) = parse_comparision(new_rest)?;

        let span = left.span.merge(right.span).merge(right.span);
        left = Expr {
            kind: ExprKind::BinOp {
                op,
                op_span,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        };
        rest = next_rest;
    }

    Ok((left, rest))
}
