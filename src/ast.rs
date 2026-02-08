// src/ast.rs
//
// All core AST types live here.
// Keep this module "dumb": structs/enums + span helpers only.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {
    Int(i64, Span),
    Str(String, Span),
    Char(char, Span),
    Bool(bool, Span),

    Var(String, Span),

    Add(Box<Expr>, Box<Expr>, Span),

    /// Comparison expression like: a > b
    Cmp(Box<Expr>, CmpOp, Box<Expr>, Span),

    /// Function call: Name(args...)
    Call(String, Vec<Expr>, Span),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IfBranch {
    pub cond: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Stmt {
    /// `let x: i32 = 5` / `Let x = 5`
    Let {
        name: String,
        ty: Option<String>,
        mutable: bool,
        expr: Expr,
        span: Span,
    },

    /// `x = 123`
    Assign {
        name: String,
        expr: Expr,
        span: Span,
    },

    /// `mute x = 1` (your A keyword for mutable variable creation)
    Mute {
        name: String,
        ty: Option<String>,
        expr: Expr,
        span: Span,
    },

    /// `If cond then { ... } ElseIf cond then { ... } Else { ... }`
    If {
        first: IfBranch,
        elseifs: Vec<IfBranch>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },

    /// Expression used as a statement (e.g. a function call)
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/* =========================
   Span helpers (public)
   ========================= */

pub fn expr_span(e: &Expr) -> Span {
    match e {
        Expr::Int(_, sp)
        | Expr::Str(_, sp)
        | Expr::Char(_, sp)
        | Expr::Bool(_, sp)
        | Expr::Var(_, sp)
        | Expr::Add(_, _, sp)
        | Expr::Cmp(_, _, _, sp)
        | Expr::Call(_, _, sp) => *sp,
    }
}

#[allow(dead_code)]
pub fn stmt_span(s: &Stmt) -> Span {
    match s {
        Stmt::Let { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::Mute { span, .. }
        | Stmt::If { span, .. } => *span,
        Stmt::Expr(e) => expr_span(e),
    }
}
