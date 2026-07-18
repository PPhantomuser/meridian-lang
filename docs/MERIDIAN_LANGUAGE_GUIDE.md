# Meridian Language Guide

Welcome to **Meridian**, a serious, compiled-first programming language designed from the ground up for the era of Human-AI Pair Programming.

This guide is the definitive resource for humans and AI agents to master reading, writing, and generating Meridian code.

---

## 1. Vision & Philosophy
Meridian bridges the gap between Python's high-level ergonomics and Rust's low-level guarantees, while maintaining 100% predictable AI legibility. 
- **Statically Typed**: Types are explicit and strictly checked.
- **Compiled First**: Meridian runs on a custom Virtual Machine (`meridian_vm`) and can be natively JIT/AOT-compiled via Cranelift (`meridian_backend_cranelift`).
- **AI Legible**: The compiler's AST and diagnostics are deterministically serializable to JSON, allowing AI agents to read the exact machine state of the code.

---

## 2. Toolchain & Usage

Meridian ships as a unified binary (`merid`).

### Running Code
To run a `.mer` file through the Virtual Machine:
```bash
cargo run --bin merid -- run path/to/script.mr
```

To natively AOT-compile and run a `.mer` file using the Cranelift backend:
```bash
cargo run --bin merid -- run --release path/to/script.mr
```

### Static Analysis
To type-check a file without executing it (AI agents should use this to validate generated code!):
```bash
cargo run --bin merid -- check path/to/script.mr
```
*(If errors exist, the CLI will output a structured JSON array of `Diagnostic` objects.)*

### Formatting & Linting
Meridian uses zero-configuration tools. All code must be formatted:
```bash
cargo run --bin merid -- fmt path/to/script.mr
cargo run --bin merid -- lint path/to/script.mr
```

---

## 3. Syntax & Features

### Variables & Mutability
Variables are declared using the `let` keyword. 
By default, variables are immutable. To make them mutable, use `let mut`.
```meridian
let x: Number = 10;
let mut counter: Number = 0;
let message: String = "Hello, AI!";
let is_active: Bool = true;
```

### Primitive Types
- `Number`: A 64-bit floating point number (f64).
- `Int`: A 64-bit signed integer.
- `String`: A UTF-8 string literal.
- `Bool`: `true` or `false`.
- `Unit`: The empty return type (similar to `()` in Rust or `void` in C).

### Arithmetic & Logic
Meridian supports standard arithmetic operators: `+`, `-`, `*`, `/`.
Equality and comparison use `==`, `!=`, `<`, `>`, `<=`, `>=`.
```meridian
let a: Number = (5.0 * 2.0) + 1.0;
```

### Structs and Enums
Meridian supports complex data types like structs and enums. They can also be made generic.

```meridian
struct Point {
    x: Number,
    y: Number,
}

enum Option<T> {
    Some(T),
    None,
}

let p: Point = Point { x: 1.0, y: 2.0 };
let opt: Option<Number> = Option::Some(42.0);
```

### Methods (Impl Blocks)
You can attach methods to both structs and enums using `impl` blocks. The first parameter should be `self` (or `&self` / `&mut self`).

```meridian
impl Point {
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
```

### Pattern Matching
Meridian features a powerful `match` expression for destructuring Enums and branching logic.

```meridian
let val: Number = match opt {
    Option::Some(v) => { v }
    Option::None => { 0.0 }
};
```

### Memory & Borrowing
Meridian ensures memory safety at compile time using a lexical borrow checker.
- **References:** `&T` for shared/immutable references, `&mut T` for exclusive/mutable references.
- **Borrowing:** Use `&x` or `&mut x` to borrow a variable.
- **Dereferencing:** Use `*x` to access the value behind a reference.
- **Rules:**
  1. You cannot have mutable aliases. If a variable is borrowed mutably (`&mut x`), it cannot be borrowed again in the same block.
  2. You can have multiple shared immutable borrows (`&x`), as long as there is no active mutable borrow.
  3. You cannot mutably borrow a variable that was declared immutable.

### Control Flow (Expression-based)
`if`, `else if`, and `else` are strictly expression-based, meaning they evaluate to a value. There is no `return` keyword needed inside them.

```meridian
let state: Number = if x == 10.0 {
    1.0
} else if x == 20.0 {
    2.0
} else {
    0.0
};
```

### Functions
Functions are declared using `fn`. They require explicit type annotations for parameters and the return type. 
The final expression in a function block is implicitly returned (there is no `return` keyword for the end of the block, though early returns may be supported later).

```meridian
fn square(n: Number) -> Number {
    n * n
}
```

### Standard Library / Built-ins
Meridian relies on standard library packages distributed via the local registry. You can import them directly:
```meridian
import "meridian-core"; // Provides Option<T>, Result<T, E>, Box<T>
import "meridian-math"; // Provides Math struct with min, max, abs, pow, etc.
```
- `print <expr>;`: A built-in statement to print a value to stdout.

---

## 4. Notes for AI Agents

If you are an AI generating Meridian code, adhere to these strict rules:
1. **Implicit Returns Only**: Do not use the `return` keyword. The last expression in a block is the return value.
2. **Type Annotations**: Always annotate variable types `let name: Type = value;`.
3. **Struct/Enum Initializers**: When writing struct or enum types, always define them completely.
4. **Structured Errors**: If you encounter an error running `merid check`, read the JSON diagnostic output. The `span` object gives you the exact byte offsets, and the `message` gives you the compiler's complaint. Adjust the code accordingly.
