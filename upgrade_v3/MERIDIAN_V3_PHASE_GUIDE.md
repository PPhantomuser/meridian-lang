# Meridian V3 — Suggested Phase Guide

This is a starter phase outline for V3. Final phase definitions should be refined in `MERIDIAN_V3_IMPLEMENTATION_PLAN.md`.

## Suggested V3 phase families

### V3.0 — V3 spec and tooling foundation
- Stabilize V3 spec pack.
- Confirm schema/versioning direction for AST/IR/diagnostics.
- Prepare planning and validation workflow.

### V3.1 — Advanced memory and borrowing model
- Introduce or refine cross-function borrowing/lifetime rules.
- Define relationship between ARC and borrowing.
- Harden safe/unsafe reference boundaries.

### V3.2 — Async and concurrency core
- Async syntax and lowering strategy.
- Runtime/executor model.
- Integration with existing thread/channel model.
- Structured concurrency and cancellation.

### V3.3 — Macro and metaprogramming foundations
- Declarative macro model.
- Expansion visibility for tools/AI.
- Reflection boundaries if any.

### V3.4 — Tooling and AI schema expansion
- LSP foundations or upgrades.
- AI-readable diagnostics evolution.
- Extended schema coverage for new constructs.

### V3.5 — Packaging and registry foundations
- Workspaces.
- Finalized manifest/lockfile semantics.
- Dependency resolution model.
- Public registry design groundwork.

### V3.6 — V3 stdlib/platform core expansion
- Collections/iterators maturity.
- Async IO scope.
- JSON/text/regex/process/path/logging candidates.

### V3.7 — Interop and deployment hardening
- C FFI hardening.
- AOT `merid build` maturity.
- Python/WASM direction where scoped for V3.

## Important note

This file is a starting guide, not the final plan.
The real plan should be rewritten and finalized in `MERIDIAN_V3_IMPLEMENTATION_PLAN.md` after the deeper V3 research/spec stage.
