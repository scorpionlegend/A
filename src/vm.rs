// src/vm.rs
//
// Minimal stack-based VM that executes Chunk bytecode.

use crate::bytecode::{Chunk, Instr, Value};
use std::io::{self, Write as _};

pub struct Vm {
    stack: Vec<Value>,
    locals: Vec<Value>,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            locals: Vec::new(),
            ip: 0,
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<(), String> {
        self.locals = vec![Value::Unit; chunk.locals.len()];
        self.ip = 0;

        while self.ip < chunk.code.len() {
            let instr = chunk.code[self.ip].clone();
            self.ip += 1;

            match instr {
                Instr::Const(v) => self.stack.push(v),

                Instr::ReadLine => {
                    let mut line = String::new();
                    // Ensure prompt flush works if user did Write("...")
                    io::stdout().flush().map_err(|e| e.to_string())?;
                    io::stdin()
                        .read_line(&mut line)
                        .map_err(|e| e.to_string())?;
                    // Strip trailing newline(s)
                    while line.ends_with('\n') || line.ends_with('\r') {
                        line.pop();
                    }
                    self.stack.push(Value::Str(line));
                }

                Instr::Print(n) => {
                    if self.stack.len() < n {
                        return Err(format!(
                            "Stack underflow: wanted to print {} values, but stack has {}",
                            n,
                            self.stack.len()
                        ));
                    }
                    let start = self.stack.len() - n;
                    let vals: Vec<Value> = self.stack.drain(start..).collect();

                    let mut out = String::new();
                    for (i, v) in vals.iter().enumerate() {
                        if i > 0 {
                            out.push(' ');
                        }
                        out.push_str(&value_to_string(v));
                    }
                    println!("{}", out);
                }

                Instr::AddInt => {
                    let b = self.stack.pop().ok_or("Stack underflow on AddInt")?;
                    let a = self.stack.pop().ok_or("Stack underflow on AddInt")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x + y)),
                        (x, y) => {
                            return Err(format!(
                                "Type error on AddInt: got {} + {}",
                                type_name(&x),
                                type_name(&y)
                            ))
                        }
                    }
                }

                Instr::LoadLocal(i) => {
                    let v = self.locals.get(i).cloned().unwrap_or(Value::Unit);
                    self.stack.push(v);
                }

                Instr::StoreLocal(i) => {
                    let v = self.stack.pop().ok_or("Stack underflow on StoreLocal")?;
                    if i >= self.locals.len() {
                        self.locals.resize(i + 1, Value::Unit);
                    }
                    self.locals[i] = v;
                }

                Instr::Jump(target) => {
                    self.ip = target;
                }

                Instr::JumpIfFalse(target) => {
                    let v = self.stack.pop().ok_or("Stack underflow on JumpIfFalse")?;
                    match v {
                        Value::Bool(false) => self.ip = target,
                        Value::Bool(true) => {}
                        other => {
                            return Err(format!(
                                "Type error: JumpIfFalse needs Bool, got {}",
                                type_name(&other)
                            ))
                        }
                    }
                }

                Instr::CmpEq => cmp_bin(self, |a, b| a == b)?,
                Instr::CmpNe => cmp_bin(self, |a, b| a != b)?,
                Instr::CmpLt => cmp_int(self, |a, b| a < b)?,
                Instr::CmpLe => cmp_int(self, |a, b| a <= b)?,
                Instr::CmpGt => cmp_int(self, |a, b| a > b)?,
                Instr::CmpGe => cmp_int(self, |a, b| a >= b)?,

                Instr::Halt => break,
            }
        }

        Ok(())
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Char(c) => c.to_string(),
        Value::Str(s) => s.clone(),
        Value::Unit => "()".to_string(),
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "Int",
        Value::Bool(_) => "Bool",
        Value::Char(_) => "Char",
        Value::Str(_) => "String",
        Value::Unit => "Unit",
    }
}

fn cmp_bin<F>(vm: &mut Vm, f: F) -> Result<(), String>
where
    F: FnOnce(Value, Value) -> bool,
{
    let b = vm.stack.pop().ok_or("Stack underflow on comparison")?;
    let a = vm.stack.pop().ok_or("Stack underflow on comparison")?;
    vm.stack.push(Value::Bool(f(a, b)));
    Ok(())
}

fn cmp_int<F>(vm: &mut Vm, f: F) -> Result<(), String>
where
    F: FnOnce(i64, i64) -> bool,
{
    let b = vm.stack.pop().ok_or("Stack underflow on comparison")?;
    let a = vm.stack.pop().ok_or("Stack underflow on comparison")?;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => {
            vm.stack.push(Value::Bool(f(x, y)));
            Ok(())
        }
        (x, y) => Err(format!(
            "Type error: int comparison needs Int/Int, got {} and {}",
            type_name(&x),
            type_name(&y)
        )),
    }
}
