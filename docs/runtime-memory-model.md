# Runtime and Memory Model

## Default approach

The default model should be memory-safe-by-default and ergonomics-first.

## Early recommendation

- ARC-style memory management by default.
- Background cycle handling if required.
- Deterministic cleanup behavior where practical.

## Advanced path

- Later opt-in ownership/borrowing or stricter low-level mode for performance-critical paths.
- Unsafe boundaries must remain explicit and lexically scoped.

## Why this direction

This keeps the language approachable while preserving a path toward systems-level performance and control.
