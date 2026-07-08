# Compiler Architecture

## Host implementation language

The first compiler/toolchain implementation should use Rust as the host language.
This does not mean Meridian is built on top of Rust as a language runtime.
It means Rust is used to implement Meridian's compiler, runtime pieces, tools, and infrastructure during early phases.

## Core pipeline

1. Source text.
2. Lexing.
3. Parsing.
4. AST construction.
5. Semantic analysis.
6. Type checking.
7. Lowering to Meridian IR.
8. Execution via VM in early versions.
9. Native backend in later phases.

## Architectural principles

- Cleanly separated stages.
- Strong diagnostics at every stage.
- Stable internal contracts between major subsystems.
- Future support for IDE/LSP reuse.
- Testability from day one.

## Early backend strategy

- Phase 1: minimal interpreter or simple execution engine.
- Phase 2/3: bytecode VM.
- Later: Cranelift backend.
- Optional later: LLVM backend.
