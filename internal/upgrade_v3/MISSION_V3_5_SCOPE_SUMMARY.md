# Mission V3.5 Scope Summary: Ecosystem Ready (Workspaces & FFI)

## 1. Scope & Goals
This phase is tightly constrained into two foundational pillars:
- **Workspaces / Packaging Layer**: Support local multi-package development using `meridian.toml`.
- **Minimal C-FFI Foundation**: Introduce `extern "C"` functions and the `unsafe` keyword to safely interact with native C code.

## 2. Out-of-Scope Items
- NO public registry or remote package publishing.
- NO generalized plugin architecture.
- NO Python/JS/WASM bridges.
- NO broad unsafe expansion beyond FFI boundaries.
- NO advanced dynamic loading model for the VM interpreter path (`libloading` is deferred).

## 3. Affected Compiler Crates/Modules
- `ast/`: Add `extern "C"` block definitions and `unsafe { ... }` block expression.
- `parser/`: Parsing logic for new keywords.
- `semantic/`: Track unsafe scope; reject FFI calls outside of `unsafe { ... }`.
- `ir/` & `backend-cranelift/`: Compilation of FFI calls to native OS calls.
- `cli/`: Resolve workspace projects via `meridian.toml`.

## 4. Exact Syntax & Design

### Extern "C"
```meridian
extern "C" {
    fn puts(s: String) -> Number;
}
```

### Unsafe Boundary
```meridian
fn safe_wrapper() {
    // Calling puts here would be a compile-time Semantic Error
    unsafe {
        puts("Hello from C!");
    }
}
```

## 5. VM and Libloading Decision
**Deferred.** We will not adopt `libloading` for the interpreter path in this phase. The VM will throw a runtime error if it attempts to execute a foreign function. FFI execution will strictly be supported via AOT/JIT (`merid run --release`) where Cranelift/LLVM can handle native system linking safely.

## 6. Acceptance Criteria
- Code with `meridian.toml` locally importing another folder compiles.
- `unsafe { ... }` correctly allows calling `extern "C"` functions, while trying to call them outside throws an error.
- FFI calls work under `merid run --release` (Cranelift).
- Test cases and documentation explicitly demonstrating workspaces and C-FFI.
