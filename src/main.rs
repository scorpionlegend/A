// src/main.rs

use clap::{Parser as ClapParser, Subcommand};
use std::{fs, path::PathBuf};

mod analysis;
mod ast;
mod bytecode;
mod compiler;
mod diag;
mod parser;
mod pipeline;
mod update;
mod vm;

#[derive(ClapParser)]
#[command(name = "a")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a .a source file or .a.byte bytecode file
    Run {
        input: PathBuf,
        /// Force running from source even if .a.byte exists
        #[arg(long)]
        fresh: bool,
    },
    /// Build only (compile) a .a source file to .a.byte bytecode
    Build {
        input: PathBuf,
        /// Output path for bytecode
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Run after building
        #[arg(long)]
        run: bool,
    },
    /// Update A from GitHub Releases
    Update {
        /// Repo in the form owner/name (overrides A_UPDATE_REPO)
        #[arg(long)]
        repo: Option<String>,
        /// Check for updates without downloading
        #[arg(long)]
        check: bool,
    },
}

fn main() {
    let args = Cli::parse();

    match args.cmd {
        Commands::Run { input, fresh } => run_cmd(input, fresh),
        Commands::Build { input, out, run } => build_cmd(input, out, run),
        Commands::Update { repo, check } => update_cmd(repo, check),
    }
}

fn run_cmd(input: PathBuf, fresh: bool) {
    if is_bytecode(&input) {
        if fresh {
            eprintln!("A_RUN: --fresh is ignored for bytecode inputs.");
        }
        run_bytecode(&input);
        return;
    }

    let bytecode_path = input.with_extension(bytecode::BYTECODE_EXT);
    if !fresh && bytecode_path.exists() {
        run_bytecode(&bytecode_path);
        return;
    }

    let src = read_text(&input);
    let file_name = input.display().to_string();
    let cleaned = pipeline::strip_line_comments_preserve_len(&src);

    let program = match parser::parse_program(&cleaned) {
        Ok(p) => p,
        Err(parse_diag) => {
            diag::render_parse_error(&src, &file_name, &parse_diag);
            std::process::exit(1);
        }
    };

    if pipeline::compile_and_maybe_run(&src, &file_name, &program, true, None).is_err() {
        std::process::exit(1);
    }
}

fn build_cmd(input: PathBuf, out: Option<PathBuf>, run: bool) {
    if is_bytecode(&input) {
        eprintln!(
            "A_BUILD: input is already bytecode ({}). Provide a .a source file.",
            bytecode::BYTECODE_SUFFIX
        );
        std::process::exit(1);
    }

    let src = read_text(&input);
    let file_name = input.display().to_string();
    let cleaned = pipeline::strip_line_comments_preserve_len(&src);

    let program = match parser::parse_program(&cleaned) {
        Ok(p) => p,
        Err(parse_diag) => {
            diag::render_parse_error(&src, &file_name, &parse_diag);
            std::process::exit(1);
        }
    };

    let out_path = out.unwrap_or_else(|| input.with_extension(bytecode::BYTECODE_EXT));
    if pipeline::compile_and_maybe_run(&src, &file_name, &program, run, Some(&out_path)).is_err() {
        std::process::exit(1);
    }
}

fn run_bytecode(path: &PathBuf) {
    let data = read_bytes(path);
    let chunk = match bytecode::decode_chunk(&data) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("A_BYTECODE: {}", msg);
            std::process::exit(1);
        }
    };

    let mut m = vm::Vm::new();
    if let Err(msg) = m.run(&chunk) {
        eprintln!("A_VM: {}", msg);
        std::process::exit(1);
    }
}

fn is_bytecode(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase().ends_with(bytecode::BYTECODE_SUFFIX))
        .unwrap_or(false)
}

fn read_text(path: &PathBuf) -> String {
    match fs::read_to_string(path) {
        Ok(s) => s.replace("\r\n", "\n"),
        Err(e) => {
            eprintln!("A_IO: failed to read {}: {}", path.display(), e);
            std::process::exit(1);
        }
    }
}

fn read_bytes(path: &PathBuf) -> Vec<u8> {
    match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("A_IO: failed to read {}: {}", path.display(), e);
            std::process::exit(1);
        }
    }
}

fn update_cmd(repo: Option<String>, check: bool) {
    if let Err(msg) = update::run(repo, check) {
        eprintln!("A_UPDATE: {}", msg);
        std::process::exit(1);
    }
}
