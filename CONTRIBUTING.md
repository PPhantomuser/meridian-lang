# Contributing to Meridian

Thank you for your interest in contributing to Meridian! Whether you're fixing a bug, adding a feature to the standard library, or improving documentation, your help is appreciated.

## Getting Started

Meridian's toolchain is built in Rust. To build the project locally, you will need the standard Rust toolchain (Cargo and `rustc`).

### 1. Clone the Repository
```bash
git clone https://github.com/your-org/meridian.git
cd meridian
```

### 2. Build the Compiler
```bash
cargo build -p meridian_cli
```

### 3. Run the Tests
Meridian's reliability relies on its extensive test suite. Before submitting any changes, ensure all tests pass:
```bash
cargo test
```

## Contribution Guidelines

To maintain code quality and ensure a smooth review process, please adhere to the following guidelines:

### 1. Formatting
Meridian enforces a strict, zero-configuration formatting policy. Before submitting a PR that modifies `.mer` code in the examples or standard library, you **must** format it using the official formatter:
```bash
cargo run -p meridian_cli -- fmt path/to/file.mer
```
*Note: Any PR containing unformatted Meridian code will fail CI.*

### 2. Diagnostics
If you are adding a new semantic or syntax check to the compiler, you must emit a structured diagnostic using the `Diagnostic` struct. Ensure you assign a unique `MER` error code (e.g., `MER0108`) and include a helpful, plain-English message and (if possible) a suggested fix.

### 3. Pull Request Process
1. **Fork the repository** and create a feature branch.
2. **Commit your changes** with descriptive commit messages.
3. **Open a Pull Request** against the `main` branch.
4. **Pass CI**: Ensure all GitHub Actions (tests, linting, formatting) pass successfully.
5. **Code Review**: A maintainer will review your code. You may be asked to make changes before it is merged.

## Proposing Major Changes (RFCs)

If you wish to propose a major change to the language syntax, semantics, or standard library, **do not open a PR with the code implementation first**. 

Instead, submit an **RFC (Request for Comments)**.
1. Read the `GOVERNANCE.md` file.
2. Copy `rfcs/0000-template.md` and draft your proposal.
3. Open a PR to the `rfcs/` directory to begin community discussion.
