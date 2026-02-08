# Diagnostic Examples for the A Language

This directory contains example programs that showcase the diagnostic messages in the A language compiler.

## Language Syntax Reference

- **Variable declaration**: `x = 1` (simple assignment creates/declares a variable)
- **Mutable variable**: `Mute x = 1` (explicitly mark as mutable)
- **Typed declaration**: `x: i32 = 1` (declare with explicit type)
- **If statement**: `If condition then { ... } Else { ... }`
- **Comparison**: `x > 5`, `x == 2`, `x != 3`
- **Function**: `Func main() { ... }`

## Files

### `valid_program.a`
A valid A program with no errors. Demonstrates correct syntax and type usage.

### Parse Errors

**`parse_error_unmatched_brace.a`**
Demonstrates a parse error when there are unmatched or extra braces. The parser will report confusion and suggest checking brace balance.

### Semantic Errors

**`a001_undeclared_variable.a` (Error Code: A001)**
Demonstrates using a variable that hasn't been declared. The error explains:
- What went wrong: variable `x` was used before being declared
- Why it's an error: the analyzer doesn't know what type or value `x` should have
- How to fix it: declare the variable first with `x = <expr>`

**`a002_type_mismatch.a` (Error Code: A002)**
Demonstrates assigning a value of one type to a variable declared as another type. The error explains:
- What went wrong: trying to assign `true` (bool) to a variable expecting an Int
- Why it's an error: type safety ensures variables hold expected types
- How to fix it: ensure the assigned value matches the variable's declared type

**`a003_add_operands.a` (Error Code: A003)**
Demonstrates using `+` with non-integer operands. The error explains:
- What went wrong: trying to add a string and an integer
- Why it's an error: `+` works only on integers in A
- How to fix it: ensure both operands are integers

**`a007_if_condition_bool.a` (Error Code: A007)**
Demonstrates using a non-boolean value in an if condition. The error explains:
- What went wrong: using `5` (an integer) as the if condition
- Why it's an error: if conditions must be yes/no questions (booleans)
- How to fix it: use a comparison operator like `x > 2` to produce a boolean

## Running the Examples

To test an example and see its error message:

```bash
cargo run --release -- run examples/a001_undeclared_variable.a
cargo run --release -- run examples/a002_type_mismatch.a
# etc.
```

Or run the valid example to see success:

```bash
cargo run --release -- run examples/valid_program.a
```
