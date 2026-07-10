<div align="center">
  <h1>🏔️ Meridian</h1>
  <p><strong>A compiled-first programming language designed for both human engineers and AI agents.</strong></p>

  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
  [![Version](https://img.shields.io/badge/version-v0.1.0-orange.svg)]()
</div>

---

## ⚡ Why Meridian?

Meridian was born out of a desire for a language that combines the **ergonomics of Python** with the **memory safety guarantees of Rust**. It is a modern, statically-typed language featuring a lightning-fast tree-walking virtual machine for local development, and an AOT (Ahead-of-Time) compiler powered by Cranelift for production deployments.

### ✨ Key Features
* **Strict Type Safety:** Catch bugs at compile time.
* **Dual Execution Modes:** Run fast in the VM, or compile to native machine code with Cranelift.
* **Agentic by Design:** First-class language primitives designed for AI tool-calling and sandboxed execution.
* **Predictable Memory Model:** Uses automatic reference counting (ARC) for seamless, GC-pause-free memory management.

---

## 💻 Code Example

Meridian syntax is designed to be clean, readable, and highly expressive.

```meridian
// A simple async fetch example
async fn fetch_data(id: Int) -> Future<Int> {
    print "Fetching user data...";
    return id * 10;
}

let result = await fetch_data(5);
print result;
```

---

## 🏗️ Architecture & Crates

The Meridian compiler is modular and written in 100% safe Rust. The core logic is split into several highly optimized crates located in the `crates/` directory:

| Component | Description |
|-----------|-------------|
| `crates/lexer` | Transforms raw source code into a token stream. |
| `crates/parser` | Builds the Abstract Syntax Tree (AST) using a recursive descent parser. |
| `crates/semantic` | Performs strict type-checking and borrow checking. |
| `crates/ir` | Lowers the AST into Meridian Intermediate Representation (MIR). |
| `crates/vm` | A blazing-fast stack-based Virtual Machine for interpreting MIR. |
| `crates/backend-cranelift`| Ahead-of-Time (AOT) compiler backend for native code generation. |

> **Note on Languages Panel**: Because the entire compiler toolchain is written in Rust, GitHub correctly identifies this repository as a Rust project. Once you start pushing Meridian (`.mer`) applications, those repositories will be classified as Meridian!

---

## 🚀 Getting Started

### Prerequisites
You must have the Rust toolchain installed to build the Meridian compiler.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation
Clone the repository and build the CLI:
```bash
git clone https://github.com/PPhantomuser/meridian-lang.git
cd meridian-lang
cargo build --release
```

### Running Your First Script
Create a file called `hello.mer`:
```meridian
print "Hello from Meridian!";
```

Run it using the Meridian CLI:
```bash
cargo run --bin merid -- run hello.mer
```

---

## 📖 Documentation

Dive deeper into the architecture and design of Meridian by exploring the `docs/` folder:
* [Meridian Language Guide](docs/MERIDIAN_LANGUAGE_GUIDE.md)
* [Compiler Architecture](docs/compiler-architecture.md)
* [Memory Model](docs/runtime-memory-model.md)

---

<div align="center">
  <i>Built with ❤️ for the open-source community.</i>
</div>
