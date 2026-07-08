# MERIDIAN — Master Design Package & Antigravity Build Prompt

> **Working codename:** *Meridian* (`.mer` source files, `merid` CLI). This is a placeholder — do a trademark/namespace search before committing to it publicly. Everywhere below, treat "Meridian" as a variable you can rename in one pass.

This document is written to be split into the individual spec files described in Section 11, and to be handed — as a whole or in parts — directly to Antigravity (Section 12) as its source of truth. It reflects one consistent, opinionated set of decisions rather than a menu of options, because an implementation agent needs a single target, not a debate.

---

## SECTION 1 — Executive Interpretation

**What you're actually asking for**, stated more sharply: a statically-typed, compiled-first language with a fast interpreted development loop, whose *entire toolchain* (not just the syntax) is designed around two simultaneous audiences — human engineers who want Python's ergonomics and Rust's guarantees, and AI agents that need a language whose structure is stable, diffable, and semantically legible enough to generate and refactor without babysitting.

That is a real, buildable project — but only if you separate three horizons clearly, because conflating them is the single most common reason ambitious language projects die:

- **v1 (0–12 months, "serious toy with teeth"):** A tree-walking-then-bytecode interpreter, a real static type checker, a real module system, a real CLI (build/run/test/fmt), and a small but *correct* standard library. Not fast. Not complete. Genuinely usable for scripts, CLIs, and small services, with a design that does not need to be thrown away later. This is the phase where you prove the syntax and semantics are good, not where you prove the language is fast.
- **v2 (12–30 months, "credible for real work"):** Native-ish performance via a real backend (Cranelift, then optionally LLVM), a package registry, an LSP good enough that people stop noticing it, async/concurrency that works under load, and the first AI-facing tooling (structured AST export, machine-readable diagnostics). This is the phase where a small company could plausibly ship a production service in it.
- **Long-term ecosystem (30+ months):** Self-hosted compiler, effect/capability system, first-class AI-agent affordances (sandboxed execution, capability-scoped tool calling as a language primitive), and community/governance structures. This is the phase where "everywhere, like Python" becomes a real question rather than a slogan.

The most important early decision is this: **v1 must not be a demo.** It must be boring, correct, and small enough to finish. Everything ambitious (effects, macros, native ML support, self-hosting) is explicitly deferred with a documented reason, not silently dropped. That distinction — deferred-with-a-plan vs. forgotten — is what this whole package is designed to protect.

---

## SECTION 2 — Design Principles

**What Meridian is:** a statically typed, memory-safe-by-default, ergonomics-first general-purpose language with a unified toolchain, designed from day one to produce code and diagnostics that are as legible to an LLM as to a human, and to let humans and AI agents co-edit the same codebase without either side silently corrupting the other's intent.

**What Meridian is not:**
- Not a research language chasing type-theoretic novelty. Every advanced feature must justify itself against a real pain point, not against "what would be interesting."
- Not a DSL for one domain (not "a language for ML," not "a language for web"). Domain support is layered on top of a general-purpose core.
- Not a syntax skin over an existing runtime (not "Python with types bolted on," not "JS with curly braces"). It needs its own semantics to actually fix the problems it targets.
- Not "safe like Rust" via borrow-checker-for-everyone in v1. Full ownership/borrowing is an opt-in advanced mode, not the default onboarding experience — this is the single biggest lesson from Rust's adoption curve.

**Why it should exist:** every incumbent language optimizes for a world without AI co-authors and treats tooling (formatter, linter, docs, package manager) as an ecosystem afterthought rather than a core deliverable. Python is safe-feeling but has a real type system bolted on after the fact and a packaging story that's still fragmented after 20 years. Rust is deeply safe but has a learning cliff that keeps it out of "first language" and "quick script" territory. Go is simple but under-powered for generic, expressive code and has a minimal error model. JavaScript/TypeScript carry decades of platform baggage. C/C++ are unsafe by default. Meridian's bet: **unify the toolchain, keep the default path shallow, make the advanced path deep, and treat AI-legibility as a first-class design constraint, not a marketing bullet.**

**The niche it should win first:** CLI tools, automation scripts, small backend services, and developer-tooling glue — the space currently split between Python (ergonomics, weak types) and Go (simple, weak generics/error model) and Rust (safe, too much upfront cost). Win there first with "as easy as Python to start, as trustworthy as Rust to scale," then expand outward.

**How it stays easy while being serious:** type inference is aggressive enough that beginners rarely write explicit types; the advanced features (ownership mode, macros, effects) are *invisible until invoked* — a file that never uses them reads like a clean scripting language. Difficulty is opt-in, not ambient, the way Kotlin or Swift keep null-safety and protocols approachable while still supporting serious systems work.

**How it deliberately bridges humans and AI:** the compiler can always emit a canonical, stable AST (JSON or S-expression) alongside source text; diagnostics always carry both a human sentence and a machine-readable code + structured payload (location, category, suggested fix as data, not just as a string); the standard library favors small, composable, pure-by-default functions over large stateful objects, because small composable units are exactly what LLMs refactor most reliably.

**Pain points this targets, explicitly:**
| Incumbent pain point | Meridian's answer |
|---|---|
| Python: weak/optional typing, packaging fragmentation (pip/poetry/conda/venv) | Real static types with inference; one CLI, one manifest, one lockfile |
| C: no memory safety, no modules, string-based build systems | Safe-by-default memory model; real module system; no headers |
| Rust: steep learning curve, borrow checker as onboarding gate | Ownership mode is opt-in; ARC-based safety by default |
| Java: verbosity, slow startup, checked-exception ceremony | Inference-heavy syntax; Result-based errors, not checked exceptions |
| JavaScript: `this`, type coercion chaos, callback/async churn | No implicit coercion; one async model (structured concurrency) from day one |
| C++: ABI chaos, undefined behavior, build system fragmentation | No UB in safe subset; single build tool; explicit unsafe boundary |
| Ruby: performance ceiling, weak tooling standardization | Bytecode VM with a real perf path; batteries-included official tooling |

**What's deliberately delayed rather than forced into v1:** macros/metaprogramming, an effect system, native tensor/ML types, self-hosting, a full borrow checker, reflection, and any plugin-based compiler architecture. Each of these is a multi-month project in its own right and each one is *safer to add later* than to retrofit — adding a stricter mode later is additive; removing an over-committed feature later is a breaking change and a trust hit.

---

## SECTION 3 — Language Product Spec

**Mission:** Make it possible for a beginner to be productive in an afternoon and for a team to trust the same language in production five years later, in a codebase that a human and an AI agent can both safely edit.

**Vision:** Meridian becomes the default choice for "I need to write something real, quickly, and not regret it later" — the CLI tool, the backend service, the automation script, the internal platform tool — and becomes the preferred language for AI coding agents because its structure resists the failure modes (silent type coercion, ambiguous scoping, un-parseable macros) that make agents unreliable in other languages.

**Target users:**
- **Beginners:** need a shallow on-ramp, good errors, no ceremony for "hello world" or a 20-line script.
- **Professionals / companies:** need types, performance headroom, a real package ecosystem, and confidence that a refactor won't silently break behavior.
- **AI agents / automated systems:** need stable, parseable structure; predictable, composable stdlib idioms; and diagnostics they can act on programmatically.
- **Research/tooling teams:** need extensibility points (IR access, plugin points in the compiler pipeline) without needing to fork the compiler.

**Key use cases:** CLI tools and scripts, automation/workflow glue, backend services and APIs, developer tooling and internal platforms, agent orchestration layers, security-sensitive utilities, local-first/offline apps.

**Non-goals (explicitly, for v1–v2):** not a browser/UI language, not a native ML/tensor-kernel language (interop with Python/C for that instead), not an embedded/real-time/no-runtime systems language (that's a "later" ambition requiring a no-GC/no-ARC mode), not a language that guarantees ABI stability across major versions before 1.0.

**Success criteria:**
- *Short term (v1):* a person with no prior exposure can write, test, and package a real CLI tool in under a day; the compiler never crashes on valid input; every error message includes a fix suggestion or a doc link.
- *Long term:* a nontrivial number of production services run on it; it appears as a first-class supported language in at least one major AI coding agent's toolchain; a community package registry has real, maintained packages beyond "hello world" ports.

**Adoption strategy:** win narrow (CLI/automation/backend-glue) before wide; make the killer first artifact the *toolchain*, not the syntax — `merid new`, `merid run`, `merid test`, `merid fmt` all working perfectly on day one builds more trust than any language feature; ship an official interop story with Python and C so teams can adopt incrementally inside existing codebases rather than rewriting; court AI-agent tooling vendors directly with the structured-AST/diagnostics story, since "a language that agents don't screw up" is a genuinely differentiated pitch right now.

**Why choose it over the incumbents:** because it's the only option that treats "an AI will also be writing this code" as a first-order design constraint instead of an afterthought, while still being pleasant for humans — most competitors are optimizing for one audience or the other, not both at once.

---

## SECTION 4 — Technical Architecture

For each subsystem: purpose, v1 approach, upgrade path, tradeoffs, and lineage.

### 4.1 Lexer
*Purpose:* turn source text into a token stream with precise spans for diagnostics.
*v1:* hand-written or `logos`-generated lexer (Rust host), UTF-8 native, significant-whitespace-free (braces + semicolons optional via automatic semicolon insertion rules borrowed from Go/Kotlin, but explicit and well-specified — not JS's ASI footguns).
*Upgrade path:* incremental/streaming lexing for LSP (re-lex only changed regions).
*Tradeoff:* brace-delimited over Python-style indentation — indentation-sensitivity is a known source of copy-paste and AI-generation bugs (misaligned blocks); braces are more robust for machine generation and diffing, at a small ergonomics cost we recoup with a great formatter.
*Lineage:* Go/Rust's ASI discipline over JavaScript's; Swift's lexer error recovery (never fail the whole file on one bad token).

### 4.2 Parser
*Purpose:* build a full AST, recovering from errors instead of aborting.
*v1:* hand-written recursive-descent + Pratt parsing for expressions (most maintainable, best error messages — this is what rustc, Roslyn, and swiftc converged on after trying parser generators).
*Upgrade path:* incremental re-parse for LSP; a stable, versioned "concrete syntax tree" (CST) layer under the AST (à la Rust Analyzer / Roslyn) so tooling (formatter, refactorer) can round-trip trivia (comments, whitespace) losslessly.
*Tradeoff:* hand-written is more work upfront than a grammar file (e.g. `pest`), but error-recovery quality and long-term maintainability win decisively — this is the single most-repeated lesson from mature compiler teams.
*Lineage:* rustc/Roslyn's recursive descent; explicitly avoiding parser-generator brittleness seen in early Ruby/PHP tooling.

### 4.3 AST
*Purpose:* typed, versioned tree representation; the thing tools and AI agents actually want to consume.
*v1:* strongly-typed AST nodes (Rust enums/structs) with stable node IDs and full source spans; a documented, versioned JSON serialization (`meridian-ast-v1`) emitted via `merid ast <file>` — this is the concrete "AI-legible structure" deliverable.
*Upgrade path:* CST-preserving edits for refactoring tools; stable node IDs that survive small edits (for diffing agent-proposed changes).
*Tradeoff:* maintaining a serialization format is ongoing work, but it's the cheapest, highest-leverage thing you can do for AI-tooling adoption — cheaper than any "AI feature," and it helps human tooling (linters, codemods) equally.

### 4.4 Semantic Analysis
*Purpose:* name resolution, scope checking, type checking, borrow/ownership checking (opt-in), effect checking (later).
*v1:* multi-pass — (1) name resolution and module linking, (2) type inference/checking (bidirectional/Hindley-Milner-flavored, with local inference and required signatures at function/module boundaries — the same "infer inside, declare at boundaries" rule Swift and OCaml both use to keep errors localized), (3) borrow/lifetime checking only inside explicitly-marked `unsafe`-adjacent "ownership blocks" (see 4.11).
*Upgrade path:* incremental/salsa-style query-based recompilation (à la rust-analyzer/Roslyn) for IDE responsiveness; effect-checking pass added as a new stage without touching earlier stages.
*Tradeoff:* a full query-based incremental architecture from day one is tempting but expensive to build correctly; v1 uses a simpler whole-module recheck and explicitly plans the query-engine rewrite for v2 once the type system is stable enough to not be constantly restructured underneath it.

### 4.5 Type System
*Purpose:* catch errors at compile time without demanding constant annotation.
*v1:* nominal structs/enums, structural traits (interfaces) for polymorphism, generics with trait bounds (Rust-flavored, not full higher-kinded types), local type inference, sum types (`enum` with payloads) as the primary error/optional mechanism (`Option<T>`, `Result<T, E>`), no implicit numeric coercion.
*Upgrade path:* associated types, const generics, limited higher-kinded polymorphism if real use cases demand it (do not add speculatively).
*Tradeoff:* skip full HKT/typeclass hierarchies (Haskell-grade) in v1 — huge implementation cost, mostly irrelevant to the target users; Rust's trait system (without full HKT) has proven this is "enough" for 95% of real code.

### 4.6 Symbol Tables
*v1:* per-module symbol tables built during name resolution, with explicit public/private visibility markers (`pub`), resolved once per compile in v1 (no incremental invalidation yet).
*Upgrade path:* per-symbol invalidation for incremental builds and LSP.

### 4.7 Intermediate Representation
*v1:* a single typed, SSA-lite IR ("MIR" — Meridian IR) generated after semantic analysis, directly interpreted by the bytecode VM (see 4.8) and later lowered to Cranelift IR.
*Upgrade path:* full SSA form with proper dominance-based optimizations once a native backend is added; a stable IR is also a natural place to hang security/lint analyses (see Section 8) that need more structure than raw AST but less than machine code.
*Tradeoff:* skip a separate "high-level IR / mid-level IR / low-level IR" pipeline (like rustc's HIR/MIR/LLVM-IR) at first — one well-designed typed IR is enough for v1–v2; split it only when a concrete optimization or backend need demands it.

### 4.8 Interpreter vs VM vs Native Compiler
*v1:* a **register-based bytecode VM** (not a tree-walker, not native) — tree-walkers are simplest to bootstrap but too slow to feel serious even for scripting use cases; native compilation on day one is too much upfront investment before the language design is validated. A bytecode VM (Python/Lua/CPython lineage, but register-based like Lua 5/CPython-post-3.11 rather than stack-based, for better performance headroom) is the correct middle point used successfully by nearly every language that later added a JIT or AOT backend.
*v2:* add a real native backend via **Cranelift** (fast to integrate, good enough codegen, used by Wasmtime/rustc's debug backend) for AOT compilation of release builds, keeping the VM for fast iteration (`merid run` uses the VM; `merid build --release` uses Cranelift).
*Later:* optional LLVM backend for maximum-optimization release builds, once the language is stable enough that the extra build complexity is worth it.
*Tradeoff:* this "VM for dev loop, real backend for release" split mirrors what Julia, and later Python (via PyPy-style approaches) and Kotlin (interpreter+K2/native) all converged on — fast iteration matters more than peak performance until the language has users.

### 4.9 Runtime Model
*v1:* single-threaded-per-task cooperative runtime with a small native thread pool for actual parallelism (Go/Tokio-flavored), a minimal runtime "core" (scheduler, allocator hooks, panic handling) written in the host language (Rust) until self-hosting.
*Upgrade path:* work-stealing scheduler for CPU-bound concurrency once async story is validated; runtime becomes a candidate for self-hosting once the language can express it.

### 4.10 Memory Management
This is a deliberately hybrid, staged decision — the single highest-leverage design choice in the whole project:
- **v1 default (safe mode):** **automatic reference counting (ARC)** with a background cycle collector for reference cycles — Swift's model. Chosen over tracing GC because it gives *deterministic* destruction (good for resource-heavy CLI/backend code, files, sockets, locks) without GC pause unpredictability, and chosen over "ownership everywhere" because ARC requires zero annotation burden for the common case — this directly protects the "easy to learn" requirement.
- **v1 opt-in (ownership mode):** explicitly-marked modules/functions can opt into a Rust-style ownership/borrow model for zero-overhead, no-ARC-refcounting hot paths (this is the "systems-grade" escape hatch, used only where profiling says it matters).
- **Later:** arena allocators as a library-level tool (not a language feature) for bulk-allocate/bulk-free patterns; explore a no-runtime "embedded" profile once the core language is proven, for real embedded/no-GC use cases.
*Lineage:* Swift's ARC for the default; Rust's ownership for the opt-in; deliberately rejecting "GC always" (Go/Java's pause unpredictability is a real complaint from latency-sensitive teams) and "borrow-check always" (Rust's documented onboarding cliff).

### 4.11 Error Handling Model
*v1:* algebraic — functions that can fail return `Result<T, E>`; `?`-style propagation operator; panics reserved for genuine programmer-error/unrecoverable conditions (index out of bounds, explicit `panic()`), never for expected failure paths. No exceptions as control flow. Diagnostics from the compiler itself always carry: human message, machine error code, source span, and (where possible) a structured suggested fix.
*Lineage:* Rust's `Result`/`?` over Java's checked exceptions (checked exceptions failed because of viral signature pollution and poor composition; `Result` composes cleanly with generics and pattern matching) and over Python/JS's uncaught-exception-by-default model.

### 4.12 Module / Package System
*v1:* directory-based modules (no header files, no `#include`), explicit `pub` visibility, one canonical manifest file (`meridian.toml`-equivalent) per package, semantic-versioned dependencies, a lockfile for reproducible builds.
*Lineage:* Rust's Cargo/module system almost directly — it is the best-regarded module+package story among mainstream languages; deliberately avoiding Python's multiple competing packaging tools and JavaScript's flat `node_modules` sprawl.

### 4.13 Build System
*v1:* one tool, `merid`, is the compiler frontend, build system, test runner, formatter invoker, and package manager entry point — no separate build-tool ecosystem (no "Makefiles," no competing build tools). Reproducible builds by default (lockfile pins exact dependency versions and, later, compiler version).
*Lineage:* Cargo/Go tooling's "one blessed tool" over C/C++'s build-system fragmentation.

### 4.14 Standard Library
*v1:* a small "core" (primitive types, collections, `Option`/`Result`, string handling, basic I/O) that is stable and near-frozen, plus a broader "std" (networking, filesystem, concurrency primitives, basic serialization) that can evolve faster. Batteries-included enough that a CLI tool or small service needs zero third-party dependencies for the basics (Python/Go's philosophy) but modular enough that core doesn't calcify.
*Lineage:* Go's small, curated stdlib over Node's "everything is npm" minimalism.

### 4.15 Testing Framework
*v1:* tests are first-class syntax (`test "description" { ... }` blocks), colocated with source by default, run via `merid test`; built-in assertion library; snapshot/golden-test support built in from the start specifically because it's the primary tool for validating compiler and stdlib behavior itself (see Section 9).
*Lineage:* Rust's `#[test]`-as-language-feature over bolted-on frameworks like Python's pytest ecosystem fragmentation.

### 4.16 Formatter / Linter
*v1:* one canonical formatter (`merid fmt`) with **no configuration options** for style (Go's `gofmt` philosophy — this single decision prevents an entire category of bikeshedding and keeps AI-generated diffs clean and reviewable); a linter (`merid lint`) with a small, curated default rule set and machine-readable output mode for tool integration.
*Lineage:* `gofmt`'s zero-config stance over Prettier/ESLint's high-configurability (configurability is a tooling-fragmentation trap).

### 4.17 Documentation Generator
*v1:* doc comments (`///`) compiled into a static docs site (`merid doc`) directly from source, including runnable doctest examples (validated by `merid test`), following rustdoc's model.

### 4.18 REPL
*v1:* a real REPL backed by the same VM used for `merid run`, supporting multi-line input, `:type`, and `:ast` introspection commands — deliberately useful for both humans exploring the language and for AI agents that want to validate a snippet before writing it to a file.

### 4.19 Debugging Strategy
*v1:* structured stack traces with source spans by default (VM mode); DWARF-ish debug info emission once the Cranelift backend exists, enabling standard native debuggers (`lldb`/`gdb`) for release builds.

### 4.20 LSP / Editor Support
*v1:* a real Language Server (`merid-lsp`) from early in Phase 3 — go-to-definition, diagnostics, hover types, basic completion — reusing the same parser/semantic analysis crates as the compiler (never a second, drifting implementation).
*Upgrade path:* incremental/query-based reanalysis (Section 4.4) for responsiveness at scale; refactoring codemods built on the CST layer.

### 4.21 FFI / Interoperability
*v1 priorities, in order:*
1. **C ABI FFI** (systems/security tooling, and the universal lowest-common-denominator for everything else) — `extern "C"` blocks, explicit `unsafe` at the boundary.
2. **Python interop** (because the AI/ML ecosystem lives there) via an embedding bridge (CPython embedding, PyO3-style) so Meridian code can call Python libraries and vice versa, without requiring Meridian to have native ML support yet.
3. **JavaScript/WASM interop** — compile Meridian to WASM as a target, enabling both browser and JS-host interop, deferred to v2 since it's not core to the initial use cases.
4. **Rust interop** — since the compiler itself is Rust-hosted initially, a Rust FFI story is a natural byproduct and useful for performance-critical stdlib pieces.
*Tradeoff:* interop is explicitly a *bridge strategy*, not a permanent crutch — the goal is to let teams adopt Meridian incrementally inside existing Python/C/Rust codebases, not to make Meridian permanently dependent on them.

### 4.22 Security Model
See Section 8 in full; architecturally, this means: an explicit, lexically-scoped `unsafe` boundary for raw memory/FFI operations, and a longer-term capability system (Section 5/7) that can restrict what a module is allowed to do (network, filesystem, process spawn) — designed into the module system from v1 even if enforcement is added later, so it isn't a breaking change to retrofit.

### 4.23 Async / Concurrency Strategy
*v1:* **structured concurrency** as the only model — `async`/`await` syntax, but tasks are always scoped to a parent (à la Kotlin coroutines / Swift's structured concurrency / Python's TaskGroup, not Go's unstructured goroutines or JS's unstructured Promises) so a function can never "leak" a background task past its own return. Channels for message passing (Go-flavored) as the primary cross-task communication primitive.
*Lineage:* deliberately choosing structured concurrency over Go's goroutine-leak-prone model and over JS's historically unstructured Promise model — this is a case where the *newer* lesson (Kotlin/Swift, ~2019-2021) is clearly better than the older ones, and adopting it from v1 avoids a painful mid-life migration (which Python, JS, and Go have all since had to partially retrofit).

### 4.24 Packaging / Distribution
*v1:* single static binary output per platform where possible (Go/Rust-style), a package registry (hosted, later) mirroring crates.io's model — one canonical registry, semantic versioning enforced, no left-pad-style single-function packages encouraged by convention/linting.

### 4.25 Future-Proofing
The IR boundary (4.7), the capability-shaped module system (4.12/4.22), and the versioned AST format (4.3) are the three architectural seams designed explicitly so new paradigms (effects, new concurrency models, new hardware backends) can be added as new passes/backends rather than rewrites. Every subsystem above is written to a stable internal interface (trait/protocol boundary) precisely so a subsystem can be swapped (e.g., VM → native backend, ARC-only → ARC+ownership) without cascading rewrites elsewhere.

---

## SECTION 5 — Language Semantics and Feature Set

Staged as **v1 / v2 / later**, with the staging reasoning stated once per group rather than repeated.

### Syntax style and philosophy
Brace-delimited, semicolon-optional (well-specified ASI, not JS-style ambiguity), inference-heavy, keyword-favoring over symbol-favoring (`fn`, `let`, `mut`, `match`) for AI/human readability. **v1.**

### Variables and mutability
`let` immutable by default, `let mut` for mutability (Rust's default, proven to reduce bugs without much ceremony). **v1.**

### Functions and closures
First-class functions, closures capture by reference under ARC (safe by default), explicit `move` keyword to force capture-by-value where needed. **v1.**

### Data structures
`struct`, `enum` (with payloads, i.e. real sum types), `trait` (interfaces with default methods, Rust/Swift-protocol-flavored). No classical inheritance (favor composition + traits — inheritance's fragility is one of the most consistently cited pain points across Java/C++/Ruby). **v1.**

### Pattern matching
`match` with exhaustiveness checking, destructuring in `let`/function params. **v1** — this is too central to error handling (`Result`/`Option`) to delay.

### Error handling and propagation
`Result<T, E>` / `Option<T>`, `?` propagation, `panic()` for unrecoverable errors. **v1.**

### Generics and type abstraction
Trait-bounded generics (monomorphized, like Rust, not type-erased like Java — better performance and better error locality). **v1**, with associated types and const generics as **v2** additions once real use cases surface.

### Traits / interfaces
As above — **v1**, no multiple inheritance of state, only of behavior (trait default methods).

### Ownership / borrowing / memory safety
ARC by default (**v1**), opt-in ownership/borrow-checked "strict mode" blocks (**v1** as a feature, but expected to be lightly used until v2 tooling matures it), full ecosystem-wide idiomatic use of ownership mode is a **v2/later** maturity milestone, not a v1 requirement.

### Imports / modules / packages
Directory-based modules, explicit `pub`, one manifest/lockfile. **v1.**

### Macros / metaprogramming
**Delayed to v2** (simple, hygienic, declarative macros only — no arbitrary compile-time code execution in v1, to avoid Rust's macro-debugging pain and C's preprocessor foot-guns). Procedural/compile-time reflection-adjacent macros are **later**, gated on real demonstrated need (e.g., serialization codegen), not spec'd until then.

### Async/await and concurrency
Structured concurrency, channels. **v1** — deferring this would force an ecosystem-wide breaking migration later (the exact mistake Python/JS/Go each made and later had to partially fix).

### Testing syntax
Built-in `test` blocks. **v1.**

### FFI syntax
`extern "C"` blocks with mandatory `unsafe` wrapping. **v1** for C; Python bridge syntax **v1** (as a stdlib/tooling feature, not new syntax); WASM target **v2**.

### Visibility / access control
`pub`, `pub(module)`-style scoped visibility. **v1.**

### Compile-time capabilities
Simple `const fn` (pure, side-effect-free functions evaluable at compile time) for constants and array sizes. **v1**, kept deliberately minimal. Broader compile-time metaprogramming is **later**, folded into the macro system decision above.

### Reflection
**No runtime reflection in v1 or v2**, by design — runtime reflection undermines both performance predictability and static-analysis reliability (a core AI-legibility goal). Structured *compile-time* introspection (the AST/IR export from Section 4.3) serves the tooling use cases reflection normally serves, without the runtime cost or the type-safety holes. Revisit only if a compelling use case survives this framing — expected answer is "no, permanently."

### Effect systems / capabilities
**Later.** The module system is shaped in v1 (Section 4.22) so that a capability annotation (e.g., "this module may perform network I/O") can be added as a new semantic-analysis pass without changing the surface syntax that already exists — this is intentionally the single most future-facing seam in the whole design, deferred because effect systems are still a genuinely unsettled research area and a bad early commitment here is expensive to undo.

### Security boundaries
Lexical `unsafe` blocks (v1), capability model (later), see Section 8.

---

## SECTION 6 — Human–AI Collaboration Strategy

**Syntax/semantics for reliable generation:** favor explicit, keyword-based constructs over dense symbolic ones (this reduces both human misreading and LLM token-level ambiguity); make invalid states genuinely unrepresentable where cheap to do so (sum types over boolean flags, `Option`/`Result` over sentinel values) — a language that makes illegal states hard to construct is a language where an AI's generated code is less likely to compile-but-misbehave.

**Diagnostics for both audiences:** every compiler error has a stable machine-readable code (`MER0142`), a structured JSON form (span, category, suggested fix as data, related spans), and a plain-English message — mirroring rustc's diagnostic model but making the JSON form a first-class, documented, versioned contract from day one rather than an internal implementation detail. This lets an AI agent parse "what's wrong" programmatically instead of regex-scraping stderr, which is what most current agent-in-the-loop coding tools are reduced to doing against other languages.

**API/module structure for safe AI composition:** encourage small, pure, composable functions in the stdlib and in idiomatic style guidance (the linter should flag large stateful "god objects" by default); favor explicit dependency passing over ambient globals/singletons, since ambient state is exactly what makes AI-driven refactors silently break things far away from the edit site.

**Static-analysis and AI-review friendliness:** the CST-preserving parse (Section 4.2/4.3) means an AI-proposed patch can be represented as a structural diff against a stable tree, not just a text diff — this is materially better for both human code review and automated review than line-based diffing, and it's a concrete, buildable feature rather than an aspiration.

**Human–AI pair programming:** the REPL's `:ast`/`:type` introspection commands and the `merid ast`/`merid check --json` CLI outputs are designed so an AI agent can validate its own generated code locally, in a tight loop, before proposing it to a human — this is the practical mechanism behind the "bridge" claim, not just a slogan.

---

## SECTION 7 — AI / LLM Ecosystem Strategy

**Python interop, and for how long:** treat Python interop as the primary on-ramp for AI/ML workloads for the entire v1–v2 horizon (roughly 24–30 months) — do not attempt native tensor/ML primitives until Meridian has enough real-world usage to justify the investment, and until it's clear what the *Meridian-idiomatic* shape of that support should be rather than copying NumPy/PyTorch APIs wholesale.

**Inference pipelines (local and remote):** v1/v2 support this via the Python bridge (call into existing Python inference stacks) plus a native HTTP/JSON stdlib good enough to call remote model APIs directly without any bridge at all — this second path is actually the more important one for "AI agent orchestration" use cases, since many agentic workflows are pure HTTP/JSON orchestration, not local tensor math.

**Data engineering and automation flows:** covered by the general-purpose stdlib (files, processes, structured data/serialization) plus the Python bridge for anything needing pandas-equivalent tooling; no bespoke "data language" features planned.

**Agent frameworks and workflow engines:** this is Meridian's strongest differentiated opportunity — structured concurrency (Section 4.23) plus algebraic error handling (Section 4.11) map unusually well onto "orchestrate several fallible async tool calls with clear cancellation semantics," which is exactly what agent frameworks need and what Python's `asyncio` and JS's Promise ecosystems both handle awkwardly. Plan a first-class "tool calling" stdlib module in v2 that models an external tool call as a typed, fallible async function — deliberately treating "call an LLM" and "call a function" as the same shape.

**Native tensor/ML support:** explicitly delayed, likely **later** (30+ months) if ever — the honest recommendation is to *not* compete with PyTorch/JAX/NumPy directly; instead, make interop so good that Meridian orchestrates around them rather than replacing them.

**Exposing model APIs, vector DBs, embeddings, tools:** as ordinary typed stdlib/ecosystem packages (an HTTP client + JSON/schema types is 90% of what's needed) rather than bespoke language features — resist the temptation to bake any specific AI vendor's API shape into the language itself.

**Staying relevant as AI evolves:** the bet is structural, not feature-chasing — a language with reliable structured diagnostics, stable AST export, and orchestration-friendly concurrency stays useful under any future AI tooling paradigm, because those are properties of the language, not integrations with today's specific AI products.

---

## SECTION 8 — Security / Systems Strategy

**Memory safety model:** safe by default (ARC, Section 4.10); all raw pointer arithmetic, manual memory management, and FFI calls require an explicit, lexically-scoped `unsafe { }` block — unsafe code cannot "leak" its unsafety into calling code without that code itself declaring `unsafe`, mirroring Rust's "unsafe is an opt-in, contained keyword" model rather than C's ambient unsafety.

**Unsafe boundary rules:** `unsafe` blocks must be minimal and are flagged by the linter if they're larger than necessary or wrap safe-representable logic; standard library functions built on `unsafe` internals must fully re-establish safety at their public boundary (Rust's "safe abstraction over unsafe implementation" pattern) — this is a documented, enforced convention, not a suggestion.

**FFI risks and patterns:** all C FFI calls are `unsafe` by construction; the Python bridge is treated as a security boundary too — data crossing it is validated/typed at the boundary, not trusted implicitly.

**Sandboxing possibilities:** design the module/capability seam (Section 4.22) so that a future sandboxed execution mode (e.g., "run this Meridian plugin with no filesystem/network access") is a natural extension of the existing module visibility system, not a bolted-on separate mechanism — directly relevant to the "AI agents executing tool code" use case, where sandboxing untrusted or AI-generated code is a real, near-term need.

**Capability-based design:** deferred implementation (Section 5's "later" effect/capability system), but reserved keyword space and module-manifest fields for it now, so it's additive later rather than breaking.

**Secure stdlib conventions:** no implicit trust of external input; parsing functions default to strict/validating modes rather than lenient ones; cryptographic primitives (once added) come only from vetted, audited implementations — never hand-rolled in the stdlib.

**Error/panic model:** panics are loud, logged, and process-terminating by default in production configuration (no silently swallowed panics) — a security-relevant choice, since silent failure is a common vector for undetected compromise or data corruption.

**Deterministic builds and reproducibility:** lockfile pins exact dependency versions; long-term goal (v2+) is pinning the exact compiler version and reproducible binary output (bit-for-bit builds), following the reproducible-builds movement that Rust, Go, and Nix have all invested in.

**Auditability and observability:** structured, machine-readable diagnostics and panics (Section 6) double as an auditability feature — the same JSON error format useful to AI tooling is also what a security/ops team wants for log aggregation.

---

## SECTION 9 — Implementation Roadmap

### Phase 0 — Research & Design Docs
**Goals:** lock down syntax, type system, and memory model decisions in writing before any code exists.
**Deliverables:** all files in Section 11's spec pack, in draft form.
**Modules:** none (docs only).
**Acceptance criteria:** every spec file exists, is internally consistent, and has no open "TBD" on a v1-scope decision.
**Risks:** analysis paralysis — mitigate with a hard timebox (recommend 3–4 weeks) and a standing rule that undecided v1 questions default to "the option this document already recommends" unless the founder actively overrides it.
**Antigravity builds:** the spec files themselves, in collaboration with the founder, plus a small set of "hello world" example programs in the intended syntax to sanity-check readability before the parser exists.
**Human review required:** all of it — this phase is where wrong decisions are cheapest to catch.

### Phase 1 — Bootstrap Prototype (interpreter)
**Goals:** lexer → parser → AST → tree-walking or simple bytecode interpreter for a minimal subset (variables, functions, `if`/`match`, structs, basic I/O — no generics, no traits, no async yet).
**Deliverables:** `merid run <file.mer>` works for the minimal subset; a golden-test suite of ~50–100 example programs with expected output.
**Modules:** `lexer/`, `parser/`, `ast/`, `interpreter/` (see Section 10).
**Acceptance criteria:** all golden tests pass; the lexer/parser never panic on malformed input (always produce a diagnostic); round-trip of the AST to/from the JSON format works.
**Risks:** scope creep into v1 features too early — mitigate by a written "Phase 1 does not include" list (generics, traits, async, macros, ownership mode) reviewed before starting.
**Antigravity builds:** all of the above, iteratively, with a design doc update whenever a design-doc assumption turns out to be wrong in practice.
**Human review required:** any point where Antigravity's implementation diverges from the spec files — this must be flagged explicitly, never silently resolved.

### Phase 2 — Usable Interpreter/Compiler MVP
**Goals:** full v1 semantics (Section 5): traits/generics, `Result`/`Option`/`?`, structured concurrency, ARC memory model, module system, `unsafe` boundary. Register-based bytecode VM replaces the tree-walker.
**Deliverables:** a real `merid` CLI (`new`, `run`, `build`, `test`), a working type checker with real diagnostics, a functioning module/import system.
**Modules:** adds `semantic/`, `ir/`, `vm/`, `runtime/`, `cli/`.
**Acceptance criteria:** a nontrivial example program (a real CLI tool with file I/O, error handling, and at least one trait/generic) builds, runs, and is tested end-to-end; type errors produce the full diagnostic contract from Section 6.
**Risks:** the type checker is the highest-complexity, highest-risk component — budget the most review time here; ARC + cycle collector correctness bugs are subtle and security-relevant, require dedicated test suites (including stress tests for cycles and concurrent access).
**Antigravity builds:** all of the above; must produce a design note whenever the type system needs a decision not already covered in `language-semantics-spec.md`.
**Human review required:** type system edge cases, memory model correctness, any new syntax not already specified.

### Phase 3 — Tooling & Ecosystem
**Goals:** `merid fmt`, `merid lint`, `merid doc`, `merid test` snapshot/golden support, LSP (`merid-lsp`), a package manifest/lockfile format, and (if hosting is available) a minimal package registry.
**Deliverables:** editor integration (at least VS Code) with go-to-definition, hover, diagnostics; a formatter with zero configuration; a working package add/install flow.
**Modules:** adds `fmt/`, `lint/`, `docgen/`, `lsp/`, `pkg/`.
**Acceptance criteria:** a second, independent developer (not the one who wrote the compiler) can install the toolchain, scaffold a project, and ship a small tool without reading compiler source code.
**Risks:** LSP responsiveness without incremental analysis (Section 4.4) may be poor on larger files — acceptable for v1, but flag as a known v2 rewrite target rather than over-engineering it now.
**Antigravity builds:** all tooling above, reusing compiler-frontend crates rather than reimplementing parsing/type-checking.
**Human review required:** package manifest/registry security model (supply-chain risk is real and worth founder sign-off) before any registry is public.

### Phase 4 — Performance & Security Hardening
**Goals:** Cranelift-backed native compilation for `--release` builds; fuzzing of the parser/type-checker; security audit of the `unsafe` boundary and FFI bridge; ARC/cycle-collector stress testing under concurrency.
**Deliverables:** `merid build --release` producing a native binary with meaningfully better performance than the VM path; a documented, triaged fuzzing pipeline; a public security-disclosure process.
**Modules:** adds `backend-cranelift/`.
**Acceptance criteria:** a defined benchmark suite shows release-mode performance within an agreed target band of comparable Go/Rust programs (not necessarily parity — set a realistic target, e.g., "within 2–4x of Rust, faster than Python/Ruby, competitive with Go" — and revisit once real numbers exist); zero known memory-safety issues in the safe subset after a fuzzing pass.
**Risks:** Cranelift integration surfacing semantic bugs the VM masked (this is common and expected — budget real time for it, don't treat it as a sign something went wrong).
**Antigravity builds:** backend integration, fuzzing harnesses, benchmark suite.
**Human review required:** any fuzzing-discovered soundness bug in the safe subset is a stop-the-line issue requiring founder sign-off on the fix before continuing other work.

### Phase 5 — AI/Runtime Expansion
**Goals:** stabilize and publicly document the AST/diagnostic JSON contracts (Section 6) as versioned, semver'd formats; ship the "tool calling" stdlib module (Section 7); explore sandboxed execution mode.
**Deliverables:** `meridian-ast-v1` and `meridian-diagnostics-v1` formats frozen and documented; at least one real integration with an external AI coding agent/tool as a proof point.
**Modules:** adds `ai/` (structured export tooling, sandboxing hooks).
**Acceptance criteria:** an external tool (not written by the core team) can consume the AST/diagnostic formats successfully from documentation alone.
**Risks:** freezing a format too early — mitigate by explicitly marking it `v1` and reserving the right to ship `v2` formats alongside it rather than breaking `v1` consumers.
**Antigravity builds:** the export tooling and its documentation/examples.
**Human review required:** the decision to freeze/version any public contract — this is a long-term commitment.

### Phase 6 — Ecosystem Growth & Community
**Goals:** governance model, contribution guidelines, package registry moderation/security policy, a public roadmap process.
**Deliverables:** `GOVERNANCE.md`, `CONTRIBUTING.md`, a public RFC process for language changes.
**Risks:** governance vacuum leading to design drift — mitigate by writing the RFC process before it's urgently needed, not after a conflict forces the issue.
**Antigravity builds:** documentation and process scaffolding; Antigravity should not unilaterally make governance decisions — flag these for the founder explicitly.
**Human review required:** all governance decisions.

---

## SECTION 10 — Repository and Folder Structure

```
meridian/
├── compiler/              # Frontend: orchestrates lexer→parser→semantic→ir per compile
│   ├── lexer/              # Tokenizer
│   ├── parser/             # Recursive-descent + Pratt parser, CST layer
│   ├── ast/                 # Typed AST definitions + versioned JSON serialization
│   ├── semantic/            # Name resolution, type checker, borrow/ownership checker
│   ├── ir/                  # MIR definition, lowering from AST, IR-level analyses
│   └── diagnostics/         # Shared diagnostic types, error codes, JSON contract
├── backend/
│   ├── vm/                  # Register-based bytecode VM (interpreter, Phase 1-2)
│   └── cranelift/           # Native codegen backend (Phase 4+)
├── runtime/                # ARC allocator hooks, scheduler, panic handling, cycle collector
├── stdlib/
│   ├── core/                 # Frozen-ish primitives: Option, Result, collections, strings
│   └── std/                  # Broader: net, fs, concurrency primitives, serialization
├── cli/                    # `merid` binary: new/run/build/test/fmt/lint/doc/pkg subcommands
├── fmt/                    # Formatter (zero-config)
├── lint/                   # Linter rules engine + default rule set
├── docgen/                 # `merid doc` static site generator + doctest runner
├── lsp/                    # Language server, reusing compiler/ crates directly
├── pkg/                    # Package manifest parsing, lockfile, registry client
├── ffi/
│   ├── c/                   # C ABI bridge + unsafe boundary enforcement
│   ├── python/              # CPython embedding bridge
│   └── wasm/                # WASM compilation target (v2+)
├── ai/                     # AST/diagnostic export tooling, sandbox hooks (Phase 5+)
├── tools/                  # Fuzzing harnesses, benchmark suite, codegen scripts
├── tests/
│   ├── golden/               # Golden/snapshot test corpora (also referenced in Section 4.15)
│   └── acceptance/           # End-to-end acceptance tests (Section 11's acceptance-tests.md)
├── examples/               # Real, runnable example programs (CLI tools, small services)
├── docs/                   # Rendered documentation, tutorials, book-style guide
└── specs/                  # The markdown spec pack from Section 11 — source of truth
```

**Why this shape supports long-term growth:** `compiler/` and `backend/` are cleanly separated so a new backend (LLVM, later) can be added without touching the frontend; `ai/` and `ffi/` are peers, not afterthoughts, signaling their permanent-citizen status; `specs/` lives in the repo (not in an external wiki) so spec and code drift is a reviewable diff, not an out-of-band problem; `stdlib/core` vs `stdlib/std` gives a concrete place to enforce the "small stable core, larger evolving std" policy from Section 4.14.

---

## SECTION 11 — Spec File Pack

Every file below should exist in `specs/` before Antigravity writes a line of compiler code, and should be treated as the literal source of truth referenced in the Section 12 prompt.

| File | Purpose | Must contain | Why it matters |
|---|---|---|---|
| `language-vision.md` | The "why" — Section 1–3 content | Mission, vision, v1/v2/long-term horizons, target users, non-goals | Prevents scope drift; the one document everything else is checked against |
| `language-design-principles.md` | The "what it is/isn't" — Section 2 | Core philosophy, pain points targeted, deliberately-delayed features list | Stops silent feature creep and silent feature abandonment |
| `language-product-spec.md` | Product framing — Section 3 | Use cases, success criteria, adoption strategy | Keeps engineering decisions tied to real users, not novelty |
| `language-syntax-spec.md` | Concrete grammar | Full EBNF-style grammar, worked examples for every construct, ASI rules | The literal contract the parser is built against |
| `language-semantics-spec.md` | Meaning, not just shape — Section 5 | Type system rules, memory model, error model, staging table (v1/v2/later) | Prevents the parser and type-checker from disagreeing on what code means |
| `compiler-architecture.md` | Section 4 subsystem breakdown | Lexer→parser→AST→semantic→IR→backend pipeline, module boundaries | The map Antigravity should never silently redraw |
| `runtime-design.md` | Runtime model — Section 4.9 | Scheduler design, panic/unwind model, VM instruction set reference | Keeps runtime and compiler-emitted IR in lockstep |
| `memory-model.md` | Section 4.10 in full detail | ARC semantics, cycle collector design, ownership-mode rules, `unsafe` interaction | The most safety-critical document in the pack; needs the most precision |
| `ffi-and-interop.md` | Section 4.21 | C ABI rules, Python bridge design, boundary-validation requirements | Prevents FFI from becoming an unaudited safety hole |
| `security-model.md` | Section 8 | `unsafe` rules, sandboxing plan, capability-system reservation, disclosure process | Non-negotiable for a language pitched at security-sensitive use cases |
| `tooling-and-lsp-roadmap.md` | Section 4.16–4.20 | Formatter/linter/doc/LSP scope per phase | Keeps tooling from being treated as "nice to have later" |
| `package-manager-spec.md` | Section 4.12–4.13, 4.24 | Manifest format, lockfile format, registry security model | Packaging fragmentation is a top-3 incumbent pain point; this must be got right once |
| `human-ai-collaboration-strategy.md` | Section 6 | Diagnostic JSON contract, AST export contract, style conventions favoring AI-legibility | The differentiated bet of the whole project — deserves its own frozen contract doc |
| `ai-ecosystem-strategy.md` | Section 7 | Python-interop policy, tool-calling stdlib design, explicit "not doing native ML yet" statement | Prevents scope creep into ML-framework territory |
| `phased-implementation-plan.md` | Section 9 | Phase goals/deliverables/acceptance criteria/risks, verbatim | Antigravity's literal execution checklist |
| `coding-standards.md` | Engineering hygiene | Host-language (Rust) style rules, commit/PR conventions, review checklist | Keeps a multi-session AI-assisted build internally consistent |
| `acceptance-tests.md` | Test contract | The golden-test corpus description, what "done" means per phase | Objective, checkable definition of done — critical for an agent-driven build |
| `risk-register.md` | Section 15 | Living risk table, owner, mitigation status | Forces risks to be tracked, not just felt |
| `decision-log.md` | Section 14 | Living decision table with revisit triggers | Prevents relitigating settled decisions without a documented reason |

---

## SECTION 12 — Master Antigravity Prompt

> Copy everything in this section verbatim into Antigravity as its primary system/mission prompt, alongside the full spec pack files from Section 11.

```
You are acting as the principal engineer and lead language implementer for a new
programming language project, codenamed "Meridian." You are not building a demo,
a toy, or a CRUD application. You are building the early foundation of a serious,
long-lived compiler and toolchain project, and you must behave accordingly:
methodically, incrementally, and with an explicit paper trail for every
consequential decision.

## 0. Ground rules

- Before writing any code, read every file in `specs/` in full. Treat these files
  as the source of truth for what to build. If the specs are ambiguous or silent
  on something you need to decide, follow the "Handling ambiguity" procedure
  below — do not guess silently and do not invent a major design change on your
  own authority.
- Never behave like a shallow CRUD-app generator. There is no database, no web
  framework, no "just wire up an API" shortcut available here. Every subsystem
  (lexer, parser, type checker, VM, etc.) must be built with the rigor of a real
  compiler engineering team, referencing the specific design lineage and
  tradeoffs documented in `compiler-architecture.md`, `language-semantics-spec.md`,
  and `memory-model.md`.
- Respect the phase boundaries in `phased-implementation-plan.md`. Do not build
  Phase 3 tooling before Phase 2's semantic analysis is solid. Do not add
  speculative Phase 5/6 features "while you're in there." If you notice a
  genuine dependency that requires pulling a later-phase task earlier, say so
  explicitly and ask before proceeding, rather than silently reordering the plan.
- Build incrementally and iteratively. Prefer many small, reviewable,
  well-tested increments over large, monolithic changes. Every increment should
  leave the codebase in a state that builds and passes its existing tests.
- Do not over-engineer v1. If a spec file marks something as "later" or "v2,"
  do not build speculative infrastructure for it now, even if it seems like it
  would be "easy while I'm here." Extensibility means leaving clean seams
  (documented interfaces, clear module boundaries) — it does not mean building
  the unused feature itself.
- Do not silently invent major design changes. If you find that a spec'd
  approach doesn't work in practice (e.g., the grammar is ambiguous as written,
  or a type-system rule doesn't compose the way the spec assumed), stop, write
  up the concrete problem and at least two candidate resolutions with tradeoffs,
  and flag it for human review before proceeding with a workaround. Do not
  quietly patch around a design problem in a way that diverges from the
  documented spec without that divergence being visible and explained.
- Preserve extensibility and future-proofing as described in the specs
  (the IR boundary, the capability-shaped module system, the versioned AST
  format) even when the immediate feature using them is simple. Do not, however,
  build extensibility for hypothetical needs not described in any spec file —
  future-proofing means respecting the seams that are already designed in, not
  inventing new ones speculatively.
- Prefer correctness, clarity, modularity, and testability over cleverness or
  premature performance optimization, in that order, for all v1/v2 work.
  Performance work is explicitly scoped to Phase 4 unless a spec file says
  otherwise.

## 1. Handling ambiguity or missing detail

When you hit a genuine gap or ambiguity in the specs:
1. State the specific question precisely (not "how should errors work" but
   "should a `match` on an enum without a wildcard arm be a hard compile error
   or a warning, given the exhaustiveness-checking requirement in
   `language-semantics-spec.md`?").
2. Propose the resolution that is most consistent with the design principles
   already stated in `language-design-principles.md` and `decision-log.md`.
3. Implement that resolution, but record it as a new row in `decision-log.md`
   (decision, alternatives considered, why chosen, when to revisit) in the same
   commit/milestone, and flag it clearly in your progress report as a decision
   made under ambiguity, so the founder can review and override it if needed.
4. Never leave an ambiguity silently unresolved in code (e.g., inconsistent
   behavior in different code paths) — resolve it explicitly, even if the
   resolution is "not yet supported, returns a clear compile error," and say so.

## 2. What to do first

1. Confirm you have read all spec files; produce a one-page summary of your
   understanding of the v1 scope boundary (what's in, what's explicitly out)
   and get that confirmed before writing code.
2. Set up the repository structure from `compiler-architecture.md` /
   the repo-structure spec, with empty but documented module stubs.
3. Begin Phase 1 exactly as scoped in `phased-implementation-plan.md`: lexer,
   then parser, then AST, then a minimal tree-walking or simple bytecode
   interpreter for the explicitly-minimal v1-subset feature list. Do not add
   generics, traits, async, or macros in Phase 1 even if they seem easy to
   include — they are explicitly out of scope for this phase.
4. Build a golden-test corpus alongside the interpreter from the very first
   working feature, not after the fact.

## 3. How to plan and report

- Before starting each phase (and before starting each major module within a
  phase), write a short mission plan: goal, concrete deliverables, acceptance
  criteria (pulled from `phased-implementation-plan.md` and
  `acceptance-tests.md`), and estimated scope.
- Report progress in terms of: what was built, what tests were added and
  whether they pass, what deviated from the spec (and why, per the ambiguity
  procedure above), and what's next. Do not report progress purely in terms of
  "lines of code" or vague completion percentages.
- Structure commits/milestones around coherent, independently reviewable units
  (e.g., "lexer: full token set + error recovery for malformed strings," not
  "various changes"). Each milestone should correspond to a testable, working
  state of the compiler.
- Maintain `decision-log.md` and `risk-register.md` as living documents,
  updated at every milestone where a relevant decision or risk changes status.

## 4. Documentation, tests, and code quality

- Every public module, type, and function gets a doc comment explaining
  purpose and, where non-obvious, rationale (referencing the relevant spec
  file section).
- Every new language feature or compiler behavior gets: unit tests for the
  isolated logic, golden/snapshot tests for end-to-end behavior, and — once
  the feature is user-facing — at least one runnable example in `examples/`.
- Acceptance tests (`acceptance-tests.md`) are the objective definition of
  "done" for a phase. Do not declare a phase complete until its acceptance
  tests pass, and do not weaken an acceptance test to make it pass — if an
  acceptance test seems wrong given what you've learned, flag it for human
  review rather than editing it unilaterally.
- Keep the codebase clean and future-proof: respect the module boundaries in
  `compiler-architecture.md`; do not reach across layers (e.g., the VM
  reaching into parser internals) when a documented interface exists for the
  purpose; if no such interface exists yet for something you need, propose one
  and flag it rather than creating an ad hoc coupling.

## 5. What must go back to the human founder

Always flag, and do not resolve unilaterally:
- Any divergence from a spec file's documented decision.
- Any new public, versioned contract (AST format, diagnostic format, package
  manifest format) before it is frozen/shipped.
- Any memory-safety or security-relevant finding, however small.
- Any governance, licensing, registry-moderation, or community-policy question.
- Any point where a phase's scope seems to require pulling in later-phase work.

Proceed methodically. Assume this project will be worked on iteratively across
many sessions — leave the codebase, the docs, and the decision/risk logs in a
state where a fresh session (human or AI) can pick up context quickly and
correctly from what's written down, not from what's only in your own working
memory.
```

---

## SECTION 13 — Task Breakdown for Antigravity

### Must-have now (v1 / Phases 1–3)
- **Bootstrap:** repo scaffold per Section 10; CI pipeline (build + test on every commit); host-language (Rust) toolchain pinning.
- **Lexer/parser/AST:** full token set incl. error-recovery for malformed literals; recursive-descent + Pratt parser for full v1 grammar; typed AST with versioned JSON export; CST trivia-preserving layer.
- **Semantic analysis/type-checker:** name resolution/module linking; bidirectional type inference with required boundary annotations; trait/generic resolution (monomorphized); exhaustiveness checking for `match`; ARC-mode borrow-adjacent checks (basic escape analysis for the opt-in ownership blocks).
- **IR/backend:** MIR definition and AST→MIR lowering; register-based bytecode VM; VM instruction set reference doc.
- **Runtime:** ARC allocator + cycle collector; structured-concurrency scheduler (single-process, thread-pool-backed); panic/unwind model with structured stack traces.
- **Stdlib:** `core` (Option/Result/collections/strings/basic I/O); `std` networking/fs/concurrency basics.
- **CLI/package manager:** `merid new/run/build/test`; manifest + lockfile format and parser.
- **Tooling:** `merid fmt` (zero-config); `merid test` incl. golden/snapshot support; `merid doc` static generator with doctests.
- **Tests/docs/examples:** golden-test corpus from day one; doc comments on every public item; 10–20 real `examples/` programs covering the full v1 feature set.
- **Security review:** `unsafe` boundary audit; FFI boundary validation review.
- **AI interop:** stable `meridian-ast-v1` JSON export; stable diagnostic JSON contract.

### Should-have soon (v2 / Phase 3–4 tail)
- LSP (`merid-lsp`) with go-to-definition/hover/diagnostics/basic completion.
- `merid lint` with curated default rule set + JSON output mode.
- Cranelift native backend for `--release` builds.
- Package registry (hosted) + supply-chain security review.
- WASM compilation target.
- Rust FFI bridge (beyond the compiler's own internal use).
- Associated types / const generics if real use cases demand them.
- Fuzzing pipeline for parser/type-checker.

### Later (advanced / Phase 5–6)
- Hygienic declarative macro system.
- Effect/capability system built on the reserved module-manifest seam.
- Sandboxed execution mode for untrusted/AI-generated code.
- Optional LLVM backend for maximum-optimization builds.
- Self-hosting (rewrite the compiler in Meridian itself).
- Native tensor/ML primitives (only if ever — treat as a long-shot, not a roadmap commitment).
- Governance/RFC process and public community infrastructure.

---

## SECTION 14 — Decision Log

| Decision | Chosen direction | Alternatives considered | Why chosen | Revisit when |
|---|---|---|---|---|
| Memory management default | ARC + cycle collector | Tracing GC (Go/Java); full ownership/borrow-check everywhere (Rust) | Deterministic destruction without Rust's onboarding cliff; matches "easy by default, powerful when opted in" | If ARC overhead proves prohibitive for a real target workload in Phase 4 benchmarks |
| Concurrency model | Structured concurrency (async/await, scoped tasks) | Unstructured goroutines (Go); unstructured Promises (classic JS) | Avoids the leak/cancellation bugs both incumbents later had to partially retrofit | Not expected to revisit; monitor real-world ergonomics in Phase 2/3 |
| v1 execution strategy | Register-based bytecode VM | Tree-walking interpreter; native compilation from day one | Best cost/benefit for validating language design before investing in a backend | Once VM perf ceiling is empirically hit in Phase 4 |
| Native backend | Cranelift first, LLVM optional later | LLVM only; custom backend | Faster integration, adequate codegen, proven in Wasmtime/rustc debug builds | If release-mode perf targets aren't met with Cranelift alone |
| Indentation vs braces | Brace-delimited | Python-style significant whitespace | More robust for machine generation/diffing; smaller AI-generation bug surface | Not expected to revisit — core identity decision |
| Error handling | `Result`/`Option` + `?`, no exceptions | Checked exceptions (Java); unchecked exceptions (Python/JS) | Composes cleanly with generics/pattern matching; avoids checked-exception signature pollution | Not expected to revisit |
| Macros in v1 | None (deferred to v2) | Full procedural macros in v1 | Avoid Rust's macro-debugging pain and C preprocessor foot-guns while language is still stabilizing | Once real stdlib serialization needs demand codegen |
| Reflection | None, ever (compile-time AST export instead) | Full runtime reflection (Java/Python-style) | Preserves static-analysis reliability and perf predictability; AST export covers the tooling use cases | Only if a compelling use case survives explicit scrutiny — expected to hold |
| Package manager | Single unified `merid` tool + one manifest/lockfile | Separate build/package tools (C/C++-style); multiple competing tools (Python-style) | Directly fixes a top incumbent pain point; proven by Cargo/Go | Not expected to revisit |
| Bootstrap host language | Rust | C++, Zig, OCaml | Memory safety for compiler infra itself; mature ecosystem (parsing/codegen crates); aligns thematically with the language's own safety story | At self-hosting milestone (Phase 6+), by design |

---

## SECTION 15 — Risk Register

| Risk | Why it matters | Probability | Impact | Mitigation | Phase affected |
|---|---|---|---|---|---|
| Type checker complexity spirals | It's the highest-complexity v1 component; a bad early design forces a rewrite | Medium | High | Freeze `language-semantics-spec.md` type rules before coding; timebox design review in Phase 0 | Phase 2 |
| ARC/cycle-collector correctness bugs | Memory-safety bugs undermine the language's core safety pitch | Medium | High | Dedicated stress-test suite incl. concurrent/cyclic cases; treat any found bug as stop-the-line | Phase 2, 4 |
| Scope creep into v1 (generics/async/macros too early) | Directly threatens "v1 must not be a demo" goal from Section 1 | High | Medium | Written "Phase 1 does not include" list, enforced in Antigravity's mission prompt | Phase 1–2 |
| LSP/tooling feels unpolished, kills early adoption | Tooling quality is the explicit adoption-strategy bet (Section 3) | Medium | High | Reuse compiler-frontend crates directly in LSP; dogfood on a second, independent developer before public release | Phase 3 |
| Cranelift integration surfaces hidden VM-semantics bugs | Common when adding a second backend; can stall Phase 4 | High | Medium | Budget explicit time for this; treat as expected work, not a failure signal | Phase 4 |
| Package registry supply-chain risk | Public registries are a documented attack vector across ecosystems (npm/PyPI incidents) | Medium | High | Founder sign-off on registry security model before public launch; namespace/verification policy from day one | Phase 3–4 |
| AST/diagnostic JSON contract frozen too early or wrong | Core AI-legibility bet; a bad early freeze creates long-term drag | Medium | Medium | Explicit `v1`/`v2` versioning from the start; commit to never breaking a frozen version, only superseding it | Phase 5 |
| Effect/capability system commitment made prematurely | Effect systems are an unsettled research area; early commitment is expensive to undo | Low (mitigated by explicit deferral) | High if it happens anyway | Keep as "later," protected by the ambiguity-handling procedure in the Antigravity prompt | Phase 5–6 |
| Founder becomes a bottleneck reviewing every Antigravity flag | Section 12's "always flag X" list could overwhelm a solo founder | Medium | Medium | Batch review points into per-milestone reports rather than real-time interruptions where possible | All phases |
| Community/governance vacuum once external contributors appear | Undocumented governance leads to design drift/conflict | Low early, rising over time | Medium | Write `GOVERNANCE.md`/RFC process in Phase 6 before it's urgently needed | Phase 6 |

---

## SECTION 16 — Questions the Founder Should Answer

**Business/vision:**
- Is the end goal a company (commercial tooling/registry/support), an open-source project seeking a foundation home, or personal/portfolio-scale — this changes registry, licensing, and governance decisions materially.
- What's the actual time/resource budget for Phases 0–4 — this document assumes a serious multi-year effort; confirm that matches reality before committing to the roadmap's pacing.
- Is "Meridian" the final name, and has a trademark/domain/package-namespace check been done?

**Technical tradeoffs:**
- Is Rust an acceptable long-term bootstrap host, or is there a strong preference (e.g., Zig, OCaml) worth reconsidering before Phase 1 starts, given how expensive it is to change later?
- Is the ARC-default/ownership-opt-in memory model an acceptable tradeoff, or is systems/embedded use a large enough near-term priority to reconsider a Rust-style ownership-by-default model despite its onboarding cost?
- What's the realistic performance bar for v1 — "good enough to feel serious for scripting/backend glue" or something closer to Go/Rust parity? This materially affects how early the Cranelift backend needs to move up in priority.

**Ecosystem/community:**
- Open-source from day one, or closed development until a stable v1 exists? This changes whether `docs/`, `specs/`, and early builds are public artifacts or internal ones.
- Who owns registry moderation and supply-chain security policy once a public package registry exists — is that a founder responsibility, a future hire, or a foundation?

**AI and security:**
- Which specific AI coding-agent ecosystems (if any) are the target integration partners for the Section 7/Phase 5 "structured AST/diagnostics" bet — naming real targets sharpens that work considerably.
- What's the acceptable risk posture for the sandboxed-execution/untrusted-code-running use case (Section 8) — is that a real near-term requirement (e.g., "run AI-agent-generated Meridian code safely") or a longer-term aspiration? This affects how early the capability system needs to move up in priority.
