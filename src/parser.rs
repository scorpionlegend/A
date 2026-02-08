// src/parser.rs

use chumsky::prelude::*;
use chumsky::text;

use crate::ast::{CmpOp, Expr, IfBranch, Program, Span, Stmt};

#[derive(Debug, Clone)]
pub struct ParseDiag {
    pub message: String,
    pub span: Span,
}

pub fn parse_program(src: &str) -> Result<Program, ParseDiag> {
    let parser = program_parser();
    let len = src.len();

    match parser.parse(src) {
        Ok(p) => Ok(p),
        Err(errs) => {
            let e = errs.into_iter().next().unwrap();
            let sp = e.span();

            // Compute human-friendly line/column and extract the source line
            let start = sp.start;
            let end = sp.end.min(len);
            let line_num = src[..start].chars().filter(|&c| c == '\n').count() + 1;
            let line_start = src[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = src[line_start..].find('\n').map(|i| line_start + i).unwrap_or(len);
            let line = &src[line_start..line_end];
            let col = start - line_start + 1; // 1-based column

            // Build a caret pointer under the offending column
            let mut pointer = String::new();
            for _ in 0..(col.saturating_sub(1)) {
                pointer.push(' ');
            }
            pointer.push('^');

            let message = format!(
                "[A_PARSE] Error: Parse error near here.\n   at line {}:{}\n\n{}\n{}\n\nHelp: The parser couldn't match the code here to the grammar.\n      Double-check braces `{{}}`, parentheses `()` and that statements are valid.\n\nExample:\nFunc main() {{\n    x = 1\n    y = x + 2\n    Print(y)\n}}\n",
                line_num, col, line, pointer
            );

            Err(ParseDiag {
                message,
                span: Span { start, end },
            })
        }
    }
}

fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    // ✅ CRLF FIX: include '\r' everywhere we treat whitespace/newlines
    let ws = one_of::<char, &str, Simple<char>>(" \t\r").repeated().ignored();
    let wsnl = one_of::<char, &str, Simple<char>>(" \t\r\n").repeated().ignored();

    let ident = text::ident().padded_by(ws.clone());

    let type_word = one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .padded_by(ws.clone());

    let type_name = just('&')
        .or_not()
        .then(type_word)
        .map(|(amp, w)| if amp.is_some() { format!("&{}", w) } else { w })
        .padded_by(ws.clone());

    let int_lit = text::int(10)
        .padded_by(ws.clone())
        .map(|s: String| s.parse::<i64>().unwrap());

    let bool_lit = choice((just("true").to(true), just("false").to(false))).padded_by(ws.clone());

    let string_lit = just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .padded_by(ws.clone());

    let char_lit = just('\'')
        .ignore_then(any().validate(|c: char, span, emit| {
            if c == '\'' || c == '\n' || c == '\r' {
                emit(Simple::custom(span, "Invalid char literal"));
                '\0'
            } else {
                c
            }
        }))
        .then_ignore(just('\''))
        .padded_by(ws.clone());

    let cmp_op = choice((
        just("==").to(CmpOp::Eq),
        just("!=").to(CmpOp::Ne),
        just("<=").to(CmpOp::Le),
        just(">=").to(CmpOp::Ge),
        just("<").to(CmpOp::Lt),
        just(">").to(CmpOp::Gt),
    ))
    .padded_by(ws.clone());

    let expr = recursive(|expr| {
        let atom = choice((
            string_lit
                .clone()
                .map_with_span(|s, sp| Expr::Str(s, Span { start: sp.start, end: sp.end })),
            char_lit
                .clone()
                .map_with_span(|c, sp| Expr::Char(c, Span { start: sp.start, end: sp.end })),
            bool_lit
                .clone()
                .map_with_span(|b, sp| Expr::Bool(b, Span { start: sp.start, end: sp.end })),
            int_lit
                .clone()
                .map_with_span(|v, sp| Expr::Int(v, Span { start: sp.start, end: sp.end })),
            ident
                .clone()
                .map_with_span(|name: String, sp| (name, Span { start: sp.start, end: sp.end }))
                .then(
                    just('(')
                        .padded_by(ws.clone())
                        .ignore_then(expr.clone().separated_by(just(',').padded_by(ws.clone())))
                        .then_ignore(just(')').padded_by(ws.clone()))
                        .or_not(),
                )
                .map_with_span(|((name, name_sp), maybe_args), sp| {
                    if let Some(args) = maybe_args {
                        Expr::Call(name, args, Span { start: sp.start, end: sp.end })
                    } else {
                        Expr::Var(name, name_sp)
                    }
                }),
            just('(')
                .padded_by(ws.clone())
                .ignore_then(expr.clone())
                .then_ignore(just(')').padded_by(ws.clone())),
        ));

        let sum = atom
            .clone()
            .then(just('+').padded_by(ws.clone()).ignore_then(atom).repeated())
            .map_with_span(|(first, rest): (Expr, Vec<Expr>), sp| {
                let span = Span { start: sp.start, end: sp.end };
                rest.into_iter().fold(first, |acc, rhs| {
                    Expr::Add(Box::new(acc), Box::new(rhs), span)
                })
            });

        sum.clone()
            .then(cmp_op.then(sum).or_not())
            .map_with_span(|(lhs, maybe): (Expr, Option<(CmpOp, Expr)>), sp| {
                if let Some((op, rhs)) = maybe {
                    Expr::Cmp(
                        Box::new(lhs),
                        op,
                        Box::new(rhs),
                        Span { start: sp.start, end: sp.end },
                    )
                } else {
                    lhs
                }
            })
    });

    // Let forms:
    let let_stmt = choice((just("Let"), just("let")))
        .padded_by(ws.clone())
        .ignore_then(choice((just("mut"), just("Mut"))).padded_by(ws.clone()).or_not())
        .then(ident.clone())
        .then_ignore(just('=').padded_by(ws.clone()))
        .then(expr.clone())
        .map_with_span(|((maybe_mut, name), expr), sp| Stmt::Let {
            name,
            ty: None,
            mutable: maybe_mut.is_some(),
            expr,
            span: Span { start: sp.start, end: sp.end },
        });

    let typed_decl = ident
        .clone()
        .then_ignore(just(':').padded_by(ws.clone()))
        .then(type_name.clone())
        .then_ignore(just('=').padded_by(ws.clone()))
        .then(expr.clone())
        .map_with_span(|((name, ty), expr), sp| Stmt::Let {
            name,
            ty: Some(ty),
            mutable: false,
            expr,
            span: Span { start: sp.start, end: sp.end },
        });

    // Mute statement
    let mute_stmt = choice((just("Mute"), just("mute")))
        .padded_by(ws.clone())
        .ignore_then(ident.clone())
        .then_ignore(just('=').padded_by(ws.clone()))
        .then(expr.clone())
        .map_with_span(|(name, expr), sp| Stmt::Mute {
            name,
            ty: None,
            expr,
            span: Span { start: sp.start, end: sp.end },
        });

    // Assignment:
    let assign_stmt = ident
        .clone()
        .then_ignore(just('=').padded_by(ws.clone()))
        .then(expr.clone())
        .map_with_span(|(name, expr), sp| Stmt::Assign {
            name,
            expr,
            span: Span { start: sp.start, end: sp.end },
        });

    // ✅ CRLF FIX: statement separators should accept '\r' too
    let newline = one_of::<char, &str, Simple<char>>("\r\n")
        .repeated()
        .at_least(1)
        .ignored();

    let stmt = recursive(|stmt| {
        let block = just('{')
            .padded_by(wsnl.clone())
            .ignore_then(stmt.clone().padded_by(wsnl.clone()).repeated())
            .then_ignore(just('}').padded_by(ws.clone())); // Use ws, not wsnl, to preserve newlines for statement separator

        let kw_if = choice((just("If"), just("if"))).padded_by(wsnl.clone());
        let kw_then = just("then").padded_by(wsnl.clone());
        let kw_elseif = choice((just("ElseIf"), just("elseif"), just("elseIf"))).padded_by(wsnl.clone());
        let kw_else = choice((just("Else"), just("else"))).padded_by(wsnl.clone());

        let cond_then_block = expr
            .clone()
            .padded_by(wsnl.clone())
            .then_ignore(kw_then.clone())
            .then(block.clone())
            .map_with_span(|(cond, body), sp| IfBranch {
                cond,
                body,
                span: Span { start: sp.start, end: sp.end },
            });

        let first_branch = kw_if.ignore_then(cond_then_block.clone());
        let elseif_branches = kw_elseif.ignore_then(cond_then_block.clone()).repeated();
        let else_block = kw_else.ignore_then(block.clone()).or_not();

        let if_stmt = first_branch
            .then(elseif_branches)
            .then(else_block)
            .map_with_span(|((first, elseifs), else_body), sp| Stmt::If {
                first,
                elseifs,
                else_body,
                span: Span { start: sp.start, end: sp.end },
            });

        choice((
            if_stmt,
            typed_decl,
            let_stmt.clone(),
            mute_stmt.clone(),
            assign_stmt.clone(),
            expr.clone().map(Stmt::Expr),
        ))
        .padded_by(ws.clone())
    });

    let stmts = stmt
        .separated_by(newline)
        .allow_trailing()
        .padded_by(ws.clone());

    let func_kw = choice((just("Func"), just("func"), just("fn")));

    func_kw
        .padded_by(wsnl.clone())
        .ignore_then(just("main").padded_by(wsnl.clone()))
        .then_ignore(just('(').padded_by(wsnl.clone()))
        .then_ignore(just(')').padded_by(wsnl.clone()))
        .then_ignore(just('{').padded_by(wsnl.clone()))
        .ignore_then(stmts.padded_by(wsnl.clone()))
        .then_ignore(just('}').padded_by(wsnl))
        .map(|stmts| Program { stmts })
}
