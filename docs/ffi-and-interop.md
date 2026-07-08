# FFI and Interoperability

## Goal

Meridian should adopt the existing world before trying to replace it.

## Priority order

1. C ABI interoperability.
2. Python interoperability.
3. Rust interoperability.
4. WASM/JavaScript interoperability later.

## Principles

- Interop is a bridge strategy, not a permanent excuse for avoiding native growth.
- Unsafe boundaries must be explicit.
- FFI must be testable and auditable.
- Interop should help teams adopt Meridian incrementally.
