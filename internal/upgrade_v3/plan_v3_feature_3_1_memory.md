# Mission V3.1 Scope Summary: Memory Layer (Borrowing)

## 1. Exact Scope & Goals
- Introduce cross-function borrowing semantics via `&T` (shared, read-only) and `&mut T` (exclusive, mutable) reference types.
- Introduce borrowing expressions (`&expr` and `&mut expr`).
- Define and enforce exclusive vs shared borrow rules natively.
- Implement strict Lexical Borrowing with *implicit elision only*.
- Define ARC interaction: taking a `&T` from an `Rc<T>` is supported seamlessly.
- Prevent mutable aliasing at compile time within the semantic checker.
- Provide AI diagnostics that fallback to suggesting ARC (`Rc`/`Arc`) when elision fails.
- Preserve compatibility with already working V1/V2/V3.0 behavior.

## 2. Out-of-Scope Items
- **NO explicit lifetime syntax.** Do not introduce `<'a>` generics. No full Rust-lifetime complexity.
- **NO async/await borrowing semantics.** (Deferred to V3.2).
- **NO macro system interactions.** (Deferred to V3.3).
- **NO registry/platform work.**
- **NO hidden runtime rewrites.**
- **NO broad type-system redesign.** Keep existing types intact.
- **NO extra syntax** beyond what is absolutely necessary for `&T` and `&mut T`.

## 3. Affected Compiler Crates/Modules
- `ast/`: Add reference types (`&T`, `&mut T`) to `Type` enum and borrow expressions (`&expr`, `&mut expr`) to `Expr`.
- `lexer/` & `parser/`: Add token parsing rules for `&` and `&mut` operators.
- `semantic/`: Core borrow checker implementation enforcing elision rules and exclusive access.
- `ir/` & `backend-cranelift/`: Lowering of references to memory pointers.
- `diagnostics/`: Ensure borrow error schemas are fully AI-legible.

## 4. Semantic Rules Introduced
1. **Exclusivity:** If a `&mut T` to a value exists in a scope, no other references (`&T` or `&mut T`) to that value may exist or be used concurrently.
2. **Elision:** A function returning a reference must take exactly one reference parameter (or `self`), implicitly tying the output lifetime to that input.
3. **Fallback:** If a user attempts a complex graph that fails elision, the compiler diagnostic must suggest wrapping the type in `Rc<T>` or `Arc<T>`.
4. **ARC Interaction:** Standard referencing works on ARC-managed types.

## 5. Risks
- **AI-legibility / Borrow Checker Complexity:** If borrow checker rules become too complex, it heavily degrades AI code generation accuracy. 
- **Mitigation:** Strict reliance on lifetime elision. If a borrow cannot be elided (e.g. escaping references, multi-parameter complex returns), the compiler rejects it immediately without prompting the user for lifetimes.
- **Silent Philosophy Changes:** Creeping into full Rust clone. Mitigation: Strict rejection of `<'a>`.

## 6. Acceptance Criteria
- `&T` and `&mut T` parse successfully.
- Semantic analysis strictly allows returning a reference tied to `self` or a single input parameter.
- Semantic analysis rejects returning references not tied to inputs (dangling references).
- Semantic analysis prevents simultaneous `&mut T` and `&T` on the same variable.
- AI diagnostics guide the user to `Rc`/`Arc` upon lifetime elision failure.
- V1/V2 code behaves identically.

## 7. Tests to be Added
- Positive unit tests for valid `&T` and `&mut T` usage across function boundaries.
- Negative tests for multiple mutable references (invalid aliasing).
- Negative tests for dangling references.
- Schema tests verifying the JSON diagnostics for borrow errors.

## 8. Docs/Tooling Changes Required
- Update `MERIDIAN_LANGUAGE_GUIDE.md` (memory section).
- Update `merid fmt` to cleanly format `&` and `&mut` tokens.
