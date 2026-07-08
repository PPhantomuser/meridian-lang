# The Meridian Programming Language - Official Guide

Welcome to the comprehensive guide to **Meridian**, a modern, statically-typed, and compiled programming language. Meridian is designed to be fast, predictable, and simple to use, borrowing syntactic cues from languages like Rust and JavaScript.

This document serves as the ultimate reference for both humans and AI agents trying to learn, write, or contribute to Meridian codebases.

---

## 1. Overview

Meridian compiles via its own internal Intermediate Representation (IR) down to machine code using a Cranelift JIT backend, making it highly performant. It currently supports strong static typing, local type inference, basic control flow loops, conditional logic, variable mutability rules, and lexical scoping.

---

## 2. Basic Syntax and Types

### Data Types
Meridian has the following built-in primitive types:
- `Number`: A 64-bit floating point number (f64).
- `String`: A standard UTF-8 string literal.
- `Bool`: A boolean value (`true` or `false`).
- `Unit` or `()`: Represents the absence of a value. Often used as the default return type of functions.

### Variables & Mutability
Variables are declared using the `let` keyword. 

By default, variables are **immutable** in Meridian. Once assigned, they cannot be reassigned.
```rust
let x: Number = 5;
x = 10; // ❌ Semantic Error: Cannot assign to immutable variable 'x'
```

To make a variable mutable, use the `mut` keyword:
```rust
let mut sum = 0; // Type inference infers 'Number'
sum = sum + 1;   // ✅ Allowed
```

### Type Inference
You do not always need to explicitly define types. Meridian's semantic analyzer can infer types from the initial assignment.
```rust
let name = "Meridian"; // Infers String
let flag = true;       // Infers Bool
```

---

## 3. Operators

### Arithmetic Operators
Meridian supports standard arithmetic on `Number` types.
- Addition: `+` (also concatenates `String` types if the left operand is a String)
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`

### Relational Operators
Used to compare expressions. Both sides must evaluate to `Number` (except for equality, which also supports `Bool` and `String`). Relational operations return a `Bool`.
- Equal to: `==`
- Not equal to: `!=`
- Less than: `<`
- Less than or equal to: `<=`
- Greater than: `>`
- Greater than or equal to: `>=`

*Note: Grouping is fully supported using parentheses `()`.*

---

## 4. Control Flow

### If / Else
Standard conditional logic using `if`, `else if`, and `else`. Note that the condition does **not** require parentheses, but the body must be enclosed in braces `{}`.

```rust
if score >= 90 {
    print "A";
} else if score >= 80 {
    print "B";
} else {
    print "C";
}
```
*Note: `if` can also act as an expression if the branches end with expressions rather than statements.*

### While Loops
Loops as long as the condition evaluates to `true`.

```rust
let mut count = 0;
while count < 10 {
    count = count + 1;
}
```

### For Loops
Meridian supports `for` loops, currently iterating exclusively over numerical ranges.
- Exclusive range (`..`): `start..end` (iterates up to `end - 1`)
- Inclusive range (`..=`): `start..=end` (iterates up to `end`)

```rust
let mut sum = 0;
// Iterates exactly 5 times (i = 1, 2, 3, 4, 5)
for i in 1..=5 {
    sum = sum + i;
}
```

### Break and Continue
Within any `while` or `for` loop, you can control iteration:
- `break;` exits the loop entirely.
- `continue;` jumps immediately to the next iteration.

```rust
for i in 1..=10 {
    if i == 5 {
        continue; // Skips printing 5
    }
    if i == 8 {
        break; // Stops the loop entirely
    }
    print i;
}
```

---

## 5. Functions

Functions in Meridian are defined using the `fn` keyword. 

### Syntax
Parameters must have explicit type annotations. The return type is specified with an arrow `->`. If omitted, it defaults to `Unit` (or `()`).

```rust
fn add_numbers(a: Number, b: Number) -> Number {
    a + b
}
```

### Implicit Returns
Meridian does not have an explicit `return` keyword. Instead, the last expression in the function body block is implicitly returned, similar to Rust. **Do not put a semicolon** at the end of the line you wish to return.

```rust
fn is_even(val: Number) -> Bool {
    let remainder = val - (val / 2) * 2;
    remainder == 0 // Implicit return
}
```

If a function returns nothing, use `()` or `Unit`:
```rust
fn do_nothing() -> () {
    let x = 5; // Statement, no return value
}
```

---

## 6. Built-in Statements

### Print
Meridian includes a built-in `print` statement for debugging and standard output.
```rust
print "Hello, world!";
print 42;
```

### Imports
You can import other `.mer` files into your program.
```rust
import "math_utils.mer";
```

---

## 7. Comments

Meridian supports two types of comments:
- **Line Comments:** Start with `//` and ignore the rest of the line.
- **Documentation Comments:** Start with `///` and are intended to document functions or modules.

```rust
/// Calculates the factorial of a number
fn factorial(n: Number) -> Number {
    // We only support positive numbers right now
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
```

---

## 8. Compiler and CLI

The Meridian toolchain provides a CLI to compile, run, and format code.

- **Run a script**: `merid run <file.mer>` (Compiles and immediately executes via Cranelift JIT).
- **Format a script**: `merid fmt <file.mer>` (Auto-formats code using the internal AST formatter).

*(Assuming binary is installed as `merid` or run via `cargo run --bin merid -- ...`)*

---

## Conclusion
Meridian is a lightweight but powerful language. With robust scoping, strong typing, mutability enforcement, and a fast JIT backend, it is an excellent tool for scripting and building structured logic.
