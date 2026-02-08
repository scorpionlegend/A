// src/compiler.rs
//
// Compile A AST -> bytecode::Chunk
// (Right now: enough to run tiny programs with Int/String/Char/Bool literals,
// addition on ints, and Print(...). Extend as you grow A.)

use crate::ast::{Expr, Program, Stmt};
use crate::bytecode::{Chunk, Instr, Value};

pub fn compile_to_bytecode(program: &Program) -> Result<Chunk, String> {
    let mut chunk = Chunk::new();

    for stmt in &program.stmts {
        compile_stmt(stmt, &mut chunk)?;
    }

    chunk.push(Instr::Halt);
    Ok(chunk)
}

fn compile_stmt(stmt: &Stmt, chunk: &mut Chunk) -> Result<(), String> {
    match stmt {
        Stmt::Let { name, expr, .. } | Stmt::Mute { name, expr, .. } => {
            // compile RHS then store into a new local slot
            compile_expr(expr, chunk)?;
            let slot = chunk.ensure_local(name);
            chunk.push(Instr::StoreLocal(slot));
            Ok(())
        }
        Stmt::Assign { name, expr, .. } => {
            // compile RHS then store into existing (or new) local slot
            compile_expr(expr, chunk)?;
            let slot = chunk.ensure_local(name);
            chunk.push(Instr::StoreLocal(slot));
            Ok(())
        }
        
        Stmt::Expr(e) => {
            // Special-case Print(...) at bytecode level
            if let Expr::Call(name, args, _) = e {
                if name.eq_ignore_ascii_case("print") {
                    for a in args {
                        compile_expr(a, chunk)?;
                    }
                    chunk.push(Instr::Print(args.len()));
                    return Ok(());
                }
            }

            // Otherwise compile expression (no side effects yet)
            compile_expr(e, chunk)?;
            Ok(())
        }
        Stmt::If { first, elseifs, else_body, .. } => {
            // Lower If/ElseIf/Else into conditional jumps.
            // Strategy:
            // 1. Compile condition, emit JumpIfFalse to else/next placeholder.
            // 2. Compile body, emit Jump to after-if placeholder.
            // 3. Patch placeholders to point at correct targets.

            // compile first condition
            compile_expr(&first.cond, chunk)?;
            let jf_pos = chunk.code.len();
            chunk.push(Instr::JumpIfFalse(0)); // placeholder

            // compile first body
            for s in &first.body {
                compile_stmt(s, chunk)?;
            }

            // after first body, jump to end
            let after_jmp_pos = chunk.code.len();
            chunk.push(Instr::Jump(0)); // placeholder to jump past remaining branches

            // patch first JumpIfFalse to point at current position (start of next branch)
            let next_branch_start = chunk.code.len();
            chunk.code[jf_pos] = Instr::JumpIfFalse(next_branch_start);

            // collect jumps that should all target the final end
            let mut end_jumps = vec![after_jmp_pos];

            // compile else-ifs
            for elseif in elseifs {
                // compile elseif condition
                compile_expr(&elseif.cond, chunk)?;
                let jf_pos = chunk.code.len();
                chunk.push(Instr::JumpIfFalse(0));

                // compile elseif body
                for s in &elseif.body {
                    compile_stmt(s, chunk)?;
                }

                // after elseif body, jump to end
                let after_jmp_pos = chunk.code.len();
                chunk.push(Instr::Jump(0));
                end_jumps.push(after_jmp_pos);

                // patch this elseif's JumpIfFalse to point at next branch start
                let next_branch_start = chunk.code.len();
                chunk.code[jf_pos] = Instr::JumpIfFalse(next_branch_start);
            }

            // compile else body if present
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    compile_stmt(s, chunk)?;
                }
            }

            // patch all end jumps to point at final end
            let final_end = chunk.code.len();
            for pos in end_jumps {
                chunk.code[pos] = Instr::Jump(final_end);
            }

            Ok(())
        }
    }
}

fn compile_expr(expr: &Expr, chunk: &mut Chunk) -> Result<(), String> {
    match expr {
        Expr::Int(v, _) => {
            chunk.push(Instr::Const(Value::Int(*v)));
            Ok(())
        }
        Expr::Bool(b, _) => {
            chunk.push(Instr::Const(Value::Bool(*b)));
            Ok(())
        }
        Expr::Char(c, _) => {
            chunk.push(Instr::Const(Value::Char(*c)));
            Ok(())
        }
        Expr::Str(s, _) => {
            chunk.push(Instr::Const(Value::Str(s.clone())));
            Ok(())
        }
        Expr::Var(name, _) => {
            // load the local slot for this variable
            // if it doesn't exist yet, that's a compile-time error (should be declared by analyzer)
            if let Some(idx) = chunk.locals.iter().position(|n| n == name) {
                chunk.push(Instr::LoadLocal(idx));
                Ok(())
            } else {
                Err(format!("Bytecode compiler: unknown variable `{}`", name))
            }
        }
        Expr::Add(a, b, _) => {
            compile_expr(a, chunk)?;
            compile_expr(b, chunk)?;
            chunk.push(Instr::AddInt);
            Ok(())
        }
        Expr::Cmp(a, op, b, _) => {
            compile_expr(a, chunk)?;
            compile_expr(b, chunk)?;
            match op {
                crate::ast::CmpOp::Eq => chunk.push(Instr::CmpEq),
                crate::ast::CmpOp::Ne => chunk.push(Instr::CmpNe),
                crate::ast::CmpOp::Lt => chunk.push(Instr::CmpLt),
                crate::ast::CmpOp::Le => chunk.push(Instr::CmpLe),
                crate::ast::CmpOp::Gt => chunk.push(Instr::CmpGt),
                crate::ast::CmpOp::Ge => chunk.push(Instr::CmpGe),
            }
            Ok(())
        }
        Expr::Call(name, args, _) => {
            // Support built-in write/print calls which return Unit (side-effect)
            if name.eq_ignore_ascii_case("print") || name.eq_ignore_ascii_case("write") {
                for a in args {
                    compile_expr(a, chunk)?;
                }
                chunk.push(Instr::Print(args.len()));
                Ok(())
            } else {
                Err(format!(
                    "Bytecode compiler: function calls not implemented yet (saw `{}`)",
                    name
                ))
            }
        }
    }
}
