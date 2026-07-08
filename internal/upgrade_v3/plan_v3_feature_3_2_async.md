# Mission V3.2 Scope Summary: Structured Async Core

## 1. Scope & Goals
- Introduce `async fn`, `.await` (postfix for chaining), and `async { ... }` blocks.
- Adopt Structured Concurrency natively (inspired by Swift/Trio).
- Implement a pluggable M:N cooperative scheduler (Event Loop) mapped onto OS threads.
- Implement cooperative cancellation via Context or implicit token checking at `.await` points.
- Allow borrowing across `.await` points safely.

## 2. Out-of-Scope Items
- **NO detached background daemon tasks.** Structured concurrency strictly forbids spawning background tasks that outlive their scope.
- Do not add standard library async HTTP or WebSockets clients (these are deferred to V4).
- Do not implement custom generators or async streams (deferred to V4).

## 3. Affected Compiler Crates/Modules
- `ast/`: Add `AsyncFn`, `AwaitExpr`, `AsyncBlock`.
- `parser/`: Parsing logic for `async` and `.await`.
- `semantic/`: Async function type inference (returning `Future<T>`), borrowing validation across suspension points.
- `ir/`: State machine lowering for async functions/blocks.
- `runtime/`: Introduce the `EventLoop` executor and task spawning scopes (`task::scope`).
- `stdlib/`: The `Future` trait and structured task primitives.

## 4. Risks (Memory, AI-legibility, Parse ambiguity)
- **Runtime Performance:** The Async runtime could degrade Cranelift JIT startup time (e.g., for `merid run`). 
- **Mitigation:** Implement lazy initialization of the event loop. The cost must be strictly zero if no async blocks are used.
- **Complexity:** State machine lowering and borrowing across `.await` requires precise lifetime tracking at suspension points.

## 5. Acceptance Criteria
- Code containing `async fn` and `.await` parses and passes semantic checks.
- A basic executor can spawn tasks within a `task::scope` and block until all child tasks finish.
- JIT startup time remains unaffected for purely synchronous code.
- Test coverage for structured task resolution, cancellation, and nested `.await` points.
- Documentation and `merid fmt` support the new async syntax.
