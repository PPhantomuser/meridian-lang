# Mission V3.4 Scope Summary: Platform Stdlib

## 1. Scope & Goals
- Expand the standard library with essential collections (`HashMap`, `HashSet`, `VecDeque`).
- Implement basic Text functionality (Regex integration, UTF-8 manipulation).
- Provide Async IO primitives (Non-blocking File IO, basic TCP/UDP sockets).
- Provide JSON core (Fast, reflection-backed or macro-backed JSON encoding and decoding).
- Incorporate testing harness natively (`#[test]`, `assert_eq!`).

## 2. Out-of-Scope Items
- **NO HTTP client/server frameworks.** (Deferred to V4).
- **NO Crypto suites.** (Deferred to V4).
- Do not implement native tensor types or auto-diff libraries.

## 3. Affected Compiler Crates/Modules
- `stdlib/`: Expansion of standard library APIs (e.g. `std::collections`, `std::io`, `std::json`).
- `runtime/`: Interfacing Async IO with the event loop from V3.2.
- `semantic/` / `ast/`: Potential adjustments for native testing attributes (`#[test]`).
- `cli/`: Wire up `merid test` command.

## 4. Risks (Memory, AI-legibility, Parse ambiguity)
- **C-FFI Interop & Memory layout:** Async IO primitives must correctly wrap lower-level OS mechanisms. Memory layout of new collections must be sound.
- **Mitigation:** Ensure careful integration with the V3.2 event loop. Thorough unit testing and memory safety bounds checking in the collections implementation.

## 5. Acceptance Criteria
- Code using `HashMap` and `HashSet` compiles and executes correctly.
- Async File IO and Socket primitives work successfully in integration tests.
- JSON serialization/deserialization behaves predictably.
- The `merid test` command correctly discovers and executes `#[test]` annotated functions, reporting passes/failures.
