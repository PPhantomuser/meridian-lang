# Meridian V3 Implementation Plan

This document outlines the strict phase-by-phase implementation plan for the Meridian V3 Advanced Core Upgrade, as derived from the reviewed V3 spec pack.

V3 is responsible for advancing Meridian beyond the V2 usability core into an advanced, serious platform language, achieving Rust/Swift/TS parity in core semantics.

## Execution Constraints
- **V3 is an upgrade, not a rewrite.**
- **Preserve safety-by-default.**
- **Ensure AI-legibility.** All ASTs and diagnostics must remain strictly machine-readable.
- **Strict Phasing.** Do not feature-dump. Execute phase by phase.

## Phase 3.0: The AST & Tooling Foundation
**Goals:** Stabilize AST, implement AI-JSON diagnostics output, setup LSP skeleton.
**Deliverables:**
- `--ai-diagnostics` compiler flag.
- Basic LSP binary (`merid-lsp`).
**Critical Out-of-Scope:** No semantic language changes (no async, borrowing, or macro changes).

## Phase 3.1: The Memory Layer (Borrowing)
**Goals:** Introduce cross-function borrowing (`&T`, `&mut T`) with strict lifetime elision.
**Deliverables:**
- Semantic analyzer borrow checking.
- Immutable/mutable enforcement across boundaries.
**Critical Out-of-Scope:** No explicit lifetime syntax (`<'a>`).

## Phase 3.2: Structured Async Core
**Goals:** Implement `async/await` keywords, task scopes, and a basic M:N runtime.
**Deliverables:**
- Async keywords and `.await`.
- `Future` trait.
- Pluggable Event loop executor.
**Critical Out-of-Scope:** No detached background daemon tasks.

## Phase 3.3: Metaprogramming (Declarative)
**Goals:** Hygienic AST-based declarative macros.
**Deliverables:**
- `macro` keyword.
- Macro expansion engine in the semantic phase.
**Critical Out-of-Scope:** No token-stream procedural macros or compiler plugins.

## Phase 3.4: Platform Stdlib
**Goals:** Core collections, JSON serialization, and Async IO wrappers.
**Deliverables:**
- `std::collections` (`HashMap`, `HashSet`, `VecDeque`).
- `std::io` (Async File and TCP/UDP).
- `std::json`.
**Critical Out-of-Scope:** No HTTP client/server frameworks or crypto suites.

## Phase 3.5: Ecosystem Ready (Workspaces & FFI)
**Goals:** Multi-crate workspace support and C-FFI stabilization.
**Deliverables:**
- Workspace `meridian.toml` and lockfile generation.
- `extern "C"` blocks and `#[repr(C)]`.
**Critical Out-of-Scope:** No public package registry implementation (deferred to V4).
