# A Language Syntax (Current)

This document describes the current, implemented syntax in A.

## File Structure

An A program must define a single `main` function:

```a
Func main() {
    Write("Hello")
}
```

Accepted spellings for the function keyword: `Func`, `func`, `fn`.

## Comments

Line comments use `//` and run to the end of the line.

```a
// This is a comment
x = 1
```

## Statements

Statements are separated by newlines (no semicolons).

### Variable Declaration / Assignment

```a
x = 1
Let x = 1
Let mut x = 1
Mute x = 1
x: i32 = 1
```

Notes:
- `Let` and `Mute` are both supported.
- `mut` after `Let` is parsed but not enforced yet.
- Type annotations like `: i32` are parsed but not enforced yet.

### If / ElseIf / Else

```a
If x > 10 then {
    Write("Big")
} ElseIf x > 5 then {
    Write("Medium")
} Else {
    Write("Small")
}
```

Notes:
- `If`, `ElseIf`, and `Else` are case-insensitive.
- `then` must be lowercase.

### Expression Statement

Any expression can be used as a statement (typically a call):

```a
Write("Hello")
```

## Expressions

### Literals

```a
123          // Int
true         // Bool
false        // Bool
'a'          // Char (single character)
"hello"      // String
```

Notes:
- Strings do not support escape sequences yet.
- Char literals must be a single non-newline character.

### Variables

```a
x
```

### Addition

```a
x + 1
```

### Comparisons

```a
x == 1
x != 1
x < 1
x <= 1
x > 1
x >= 1
```

Only a single comparison is supported per expression (no chaining like `1 < x < 3`).

### Function Calls

```a
Write("Hello")
Print(x, y)
```

Currently supported built-ins (case-insensitive):
- `Write(...)`
- `Print(...)`

## Current Limitations

- Only `Func main()` is supported (no user-defined functions yet).
- No modules/imports yet.
- One global scope in `main` (blocks do not create new scopes).
- Type annotations and `mut` are parsed but not enforced.
