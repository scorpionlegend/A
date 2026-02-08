// src/pipeline.rs

use crate::{analysis, bytecode, compiler, diag, vm};
use crate::ast::Program;
use std::fs;
use std::path::Path;

pub fn strip_line_comments_preserve_len(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());

    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && (i + 1) < bytes.len() && bytes[i + 1] == b'/' {
            out.push(' ');
            out.push(' ');
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

pub fn compile_and_maybe_run(
    src: &str,
    file_name: &str,
    program: &Program,
    mode_run: bool,
    emit_path: Option<&Path>,
) -> Result<(), ()> {
    // 1) Analyze (A-native lessons)
    match analysis::analyze(program) {
        Ok(_ar) => {}
        Err(errors) => {
            for e in &errors {
                diag::render_lesson_error(src, file_name, e);
            }
            return Err(());
        }
    }

    // 2) Compile to bytecode
    let chunk = match compiler::compile_to_bytecode(program) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("A_BACKEND: bytecode compiler error: {}", msg);
            return Err(());
        }
    };

    // 3) Emit bytecode if requested
    if let Some(path) = emit_path {
        let data = match bytecode::encode_chunk(&chunk) {
            Ok(d) => d,
            Err(msg) => {
                eprintln!("A_BYTECODE: {}", msg);
                return Err(());
            }
        };
        if let Err(e) = fs::write(path, data) {
            eprintln!("A_BUILD: failed to write bytecode: {}", e);
            return Err(());
        }
        println!("Build succeeded. Wrote bytecode to {}", path.display());
    }

    // 4) Run VM if requested
    if mode_run {
        let mut m = vm::Vm::new();
        if let Err(msg) = m.run(&chunk) {
            eprintln!("A_VM: {}", msg);
            return Err(());
        }
    } else if emit_path.is_none() {
        println!("Build succeeded.");
    }

    Ok(())
}
