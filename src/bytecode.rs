// src/bytecode.rs
//
// Minimal bytecode model for A.
// You can extend this as you add features (strings, locals, jumps, etc.).

use serde::{Deserialize, Serialize};

pub const BYTECODE_VERSION: u32 = 1;
pub const BYTECODE_EXT: &str = "a.byte";
pub const BYTECODE_SUFFIX: &str = ".a.byte";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Char(char),
    Str(String),
    Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Instr {
    /// Push a constant onto the stack
    Const(Value),

    /// Read a line from stdin (text input), push as Value::Str
    ReadLine,

    /// Pop and print N values (simple version: prints with spaces)
    Print(usize),

    /// Arithmetic (expects Int, Int)
    AddInt,

    /// Load a local variable slot onto the stack
    LoadLocal(usize),

    /// Store top of stack into a local variable slot
    StoreLocal(usize),

    /// Jump to absolute instruction index
    Jump(usize),

    /// Pop a boolean and jump to index if it is false
    JumpIfFalse(usize),

    /// Comparison operations (pop right, pop left, push Bool)
    CmpEq,
    CmpNe,
    CmpLt,
    CmpLe,
    CmpGt,
    CmpGe,

    /// Halt program
    Halt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Chunk {
    pub code: Vec<Instr>,
    pub consts: Vec<Value>,
    pub locals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeFile {
    pub version: u32,
    pub chunk: Chunk,
}

impl BytecodeFile {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            version: BYTECODE_VERSION,
            chunk,
        }
    }
}

pub fn encode_chunk(chunk: &Chunk) -> Result<Vec<u8>, String> {
    let file = BytecodeFile::new(chunk.clone());
    bincode::serialize(&file).map_err(|e| e.to_string())
}

pub fn decode_chunk(bytes: &[u8]) -> Result<Chunk, String> {
    let file: BytecodeFile = bincode::deserialize(bytes).map_err(|e| e.to_string())?;
    if file.version != BYTECODE_VERSION {
        return Err(format!(
            "Unsupported bytecode version {} (expected {})",
            file.version, BYTECODE_VERSION
        ));
    }
    Ok(file.chunk)
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            consts: Vec::new(),
            locals: Vec::new(),
        }
    }

    pub fn push(&mut self, i: Instr) {
        self.code.push(i);
    }

    #[allow(dead_code)]
    pub fn add_const(&mut self, v: Value) -> usize {
        self.consts.push(v);
        self.consts.len() - 1
    }

    pub fn ensure_local(&mut self, name: &str) -> usize {
        if let Some(i) = self.locals.iter().position(|n| n == name) {
            i
        } else {
            self.locals.push(name.to_string());
            self.locals.len() - 1
        }
    }
}
