# Mission V3.5 Scope Summary: Ecosystem Ready (Workspaces & FFI)

## 1. Scope & Goals
- **Workspaces / Packaging Layer**: Support local multi-package development using `meridian.toml`. Add minimal package/workspace resolution required for local development.
- **Minimal C-FFI Foundation**: Introduce `extern "C"` functions and the `unsafe` keyword to safely interact with native C code. Add minimal extern "C" FFI surface and explicit `unsafe { ... }` boundary for raw pointer/native calls only.
- **Testing**: Add tests, docs, and examples for workspaces and basic C-FFI.

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

## 4. Risks (Memory, AI-legibility, Parse ambiguity)
- **Ecosystem Stall:** Missing ecosystem libraries might stall adoption post-V3. 
- **Mitigation:** Invest heavily in C-FFI so users can wrap existing C/C++ libraries easily. 
- **Safety Holes:** FFI introduces raw pointers which violate Meridian's safety model.
- **Mitigation:** Explicitly constrain FFI calls and pointer dereferencing to `unsafe { ... }` blocks. Ensure safety guarantees hold completely outside of those blocks.

## 5. Exact Syntax & Design

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

## 6. VM and Libloading Decision
**Deferred.** We will not adopt `libloading` for the interpreter path in this phase. The VM will throw a runtime error if it attempts to execute a foreign function. FFI execution will strictly be supported via AOT/JIT (`merid run --release`) where Cranelift/LLVM can handle native system linking safely.

## 7. Acceptance Criteria
- Code with `meridian.toml` locally importing another folder compiles.
- `unsafe { ... }` correctly allows calling `extern "C"` functions, while trying to call them outside throws an error.
- FFI calls work under `merid run --release` (Cranelift).
- Test cases and documentation explicitly demonstrating workspaces and C-FFI.
