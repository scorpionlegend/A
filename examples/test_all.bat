@echo off
REM Test all diagnostic examples
REM Run from the project root: examples\test_all.bat

echo.
echo === Testing Valid Program ===
cargo run --release -- run examples\valid_program.a
echo.

echo.
echo === Testing Parse Error (Unmatched Brace) ===
cargo run --release -- run examples\parse_error_unmatched_brace.a
echo.

echo.
echo === Testing A001 (Undeclared Variable) ===
cargo run --release -- run examples\a001_undeclared_variable.a
echo.

echo.
echo === Testing A002 (Type Mismatch) ===
cargo run --release -- run examples\a002_type_mismatch.a
echo.

echo.
echo === Testing A003 (Add Operands) ===
cargo run --release -- run examples\a003_add_operands.a
echo.

echo.
echo === Testing A007 (If Condition Bool) ===
cargo run --release -- run examples\a007_if_condition_bool.a
echo.
