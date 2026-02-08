// src/analysis.rs

use std::collections::HashMap;

use crate::ast::{expr_span, Expr, IfBranch, Program, Span, Stmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AType {
    Int,
    Bool,
    Char,
    Str,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AError {
    pub span: Span,
    pub code: String,
    pub title: String,
    pub mental_model: String,
    pub help: Vec<String>,
    pub example: String,
    pub backend: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnalysisResult {
    pub locals: HashMap<String, usize>,
    pub local_types: Vec<AType>,
}

pub fn analyze(program: &Program) -> Result<AnalysisResult, Vec<AError>> {
    // We'll walk statements sequentially, collecting locals and inferred types.
    let mut locals: HashMap<String, usize> = HashMap::new();
    let mut local_types: Vec<AType> = Vec::new();

    let mut errors: Vec<AError> = Vec::new();

    for s in &program.stmts {
        match s {
            Stmt::Let { name, expr, .. } | Stmt::Mute { name, expr, .. } => {
                // infer the expression type using the current symbol table
                let ty = infer_expr_type(expr, &locals, &local_types);
                let idx = local_types.len();
                locals.insert(name.clone(), idx);
                local_types.push(ty);

                // still run deeper checks inside the expression
                check_expr(expr, &locals, &local_types, &mut errors);
            }

            Stmt::Assign { name, expr, .. } => {
                // In A, a bare assignment `x = <expr>` declares `x` if it doesn't exist yet.
                if let Some(&idx) = locals.get(name) {
                    // existing variable: type-check the assignment
                    let expected = &local_types[idx];
                    let found = infer_expr_type(expr, &locals, &local_types);
                    if *expected != AType::Unknown && found != AType::Unknown && *expected != found {
                        errors.push(a002_assign_type_mismatch(expr_span(expr), expected.clone(), found));
                    }
                } else {
                    // treat as declaration: infer type and register the variable
                    let ty = infer_expr_type(expr, &locals, &local_types);
                    let idx = local_types.len();
                    locals.insert(name.clone(), idx);
                    local_types.push(ty);
                }

                check_expr(expr, &locals, &local_types, &mut errors);
            }

            Stmt::If { first, elseifs, else_body, .. } => {
                check_branch_with_ctx(first, &locals, &local_types, &mut errors);
                for br in elseifs {
                    check_branch_with_ctx(br, &locals, &local_types, &mut errors);
                }
                if let Some(body) = else_body {
                    for s in body {
                        // recursively analyze inner statements (simple approach)
                        match s {
                            Stmt::Let { name, expr, .. } | Stmt::Mute { name, expr, .. } => {
                                let ty = infer_expr_type(expr, &locals, &local_types);
                                let idx = local_types.len();
                                locals.insert(name.clone(), idx);
                                local_types.push(ty);
                                check_expr(expr, &locals, &local_types, &mut errors);
                            }
                            Stmt::Assign { name, expr, .. } => {
                                if let Some(&idx) = locals.get(name) {
                                    let expected = &local_types[idx];
                                    let found = infer_expr_type(expr, &locals, &local_types);
                                    if *expected != AType::Unknown && found != AType::Unknown && *expected != found {
                                        errors.push(a002_assign_type_mismatch(expr_span(expr), expected.clone(), found));
                                    }
                                } else {
                                    errors.push(a001_undeclared_variable(expr_span(expr), name.clone()));
                                }
                                check_expr(expr, &locals, &local_types, &mut errors);
                            }
                            Stmt::Expr(e) => check_expr(e, &locals, &local_types, &mut errors),
                            _ => {}
                        }
                    }
                }
            }

            Stmt::Expr(e) => {
                check_expr(e, &locals, &local_types, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(AnalysisResult { locals, local_types })
    } else {
        Err(errors)
    }
}

fn check_branch_with_ctx(br: &IfBranch, locals: &HashMap<String, usize>, local_types: &Vec<AType>, errors: &mut Vec<AError>) {
    let ty = infer_expr_type(&br.cond, locals, local_types);

    if !matches!(ty, AType::Bool | AType::Unknown) {
        let sp = expr_span(&br.cond);
        errors.push(a007_if_condition_must_be_bool(sp));
    }

    for s in &br.body {
        match s {
            Stmt::Let { expr, .. } | Stmt::Mute { expr, .. } => check_expr(expr, locals, local_types, errors),
            Stmt::Assign { expr, .. } => check_expr(expr, locals, local_types, errors),
            Stmt::Expr(e) => check_expr(e, locals, local_types, errors),
            _ => {}
        }
    }
}

fn check_expr(e: &Expr, locals: &HashMap<String, usize>, local_types: &Vec<AType>, errors: &mut Vec<AError>) {
    match e {
        Expr::Add(a, b, _) => {
            check_expr(a, locals, local_types, errors);
            check_expr(b, locals, local_types, errors);
            let ta = infer_expr_type(a, locals, local_types);
            let tb = infer_expr_type(b, locals, local_types);
            if !(matches!(ta, AType::Int | AType::Unknown) && matches!(tb, AType::Int | AType::Unknown)) {
                errors.push(a003_add_operands_must_be_int(expr_span(e), ta, tb));
            }
        }
        Expr::Cmp(a, _, b, _) => {
            check_expr(a, locals, local_types, errors);
            check_expr(b, locals, local_types, errors);
        }
        Expr::Call(_, args, _) => {
            for a in args {
                check_expr(a, locals, local_types, errors);
            }
        }
        Expr::Var(_, _) => {}
        _ => {}
    }
}

fn infer_expr_type(e: &Expr, locals: &HashMap<String, usize>, local_types: &Vec<AType>) -> AType {
    match e {
        Expr::Int(_, _) => AType::Int,
        Expr::Bool(_, _) => AType::Bool,
        Expr::Char(_, _) => AType::Char,
        Expr::Str(_, _) => AType::Str,
        Expr::Cmp(_, _, _, _) => AType::Bool,
        Expr::Add(a, b, _) => {
            let ta = infer_expr_type(a, locals, local_types);
            let tb = infer_expr_type(b, locals, local_types);
            match (ta, tb) {
                (AType::Int, AType::Int) => AType::Int,
                (AType::Unknown, AType::Int) | (AType::Int, AType::Unknown) => AType::Unknown,
                _ => AType::Unknown,
            }
        }
        Expr::Var(name, _) => {
            if let Some(&idx) = locals.get(name) {
                local_types.get(idx).cloned().unwrap_or(AType::Unknown)
            } else {
                AType::Unknown
            }
        }
        Expr::Call(_, args, _) => {
            if args.is_empty() {
                AType::Unknown
            } else {
                let _ = args.iter().map(|a| infer_expr_type(a, locals, local_types)).collect::<Vec<_>>();
                AType::Unknown
            }
        }
    }
}

fn a007_if_condition_must_be_bool(span: Span) -> AError {
    AError {
        span,
        code: "A007".to_string(),
        title: "If condition must be true/false (bool)".to_string(),
        mental_model:
            "`If` asks a yes/no question. The condition must already be yes/no.".to_string(),
        help: vec![
            "Option 1: Compare to produce a boolean (example: `age > 18`).".to_string(),
            "Option 2: Use `==` / `!=` to test equality.".to_string(),
        ],
        example: r#"Func main() {
    age: i32 = 20
    If age > 18 then {
        Print("Adult")
    } Else {
        Print("Not adult")
    }
}"#
        .to_string(),
        backend: None,
    }
}

fn a001_undeclared_variable(span: Span, name: String) -> AError {
    AError {
        span,
        code: "A001".to_string(),
        title: format!("Use of undeclared variable '{}'", name),
        mental_model: "You used a variable that hasn't been declared yet.".to_string(),
        help: vec![format!("Declare it first: `{} = <expr>`", name)],
        example: format!("Func main() {{\n    {} = 1\n}}", name),
        backend: None,
    }
}

fn a002_assign_type_mismatch(span: Span, expected: AType, found: AType) -> AError {
    AError {
        span,
        code: "A002".to_string(),
        title: "Type mismatch in assignment".to_string(),
        mental_model: format!("The value assigned has type {:?} but the variable expects {:?}.", found, expected),
        help: vec!["Ensure the assigned value matches the variable's type.".to_string()],
        example: "Example: `x = 1 + 2` (assigning int to int)".to_string(),
        backend: None,
    }
}

fn a003_add_operands_must_be_int(span: Span, left: AType, right: AType) -> AError {
    AError {
        span,
        code: "A003".to_string(),
        title: "Add operands must be integers".to_string(),
        mental_model: format!("`+` expects integer operands but found {:?} and {:?}.", left, right),
        help: vec!["Ensure both sides are integers (e.g., `1 + 2`), or convert values explicitly.".to_string()],
        example: "Example: `x = 1 + 2`".to_string(),
        backend: None,
    }
}
