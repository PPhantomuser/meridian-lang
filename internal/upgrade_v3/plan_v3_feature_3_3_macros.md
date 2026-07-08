# Mission V3.3 Scope Summary: Metaprogramming (Declarative)

## 1. Scope & Goals
- Introduce hygienic, AST-safe declarative macros (using the `macro` keyword).
- Ensure that expanded macros remain visible as valid AST to tooling (LSP, AI schemas).
- Implement compile-time reflection foundations (e.g., `Type::fields()`) to reduce reliance on macros for serialization boilerplate.

## 2. Out-of-Scope Items
- **NO Procedural Macros / Token-stream plugins.** They obscure code meaning from AI and LSPs.
- Do not implement custom syntax extensions beyond what AST-safe macros allow.
- Do not implement runtime reflection (only compile-time reflection).

## 3. Affected Compiler Crates/Modules
- `ast/`: Add `MacroDef` and `MacroCall` nodes.
- `parser/`: Parsing logic for declarative macros.
- `semantic/`: The macro expansion engine operating at the AST level, ensuring hygiene and enforcing recursion limits. Introduce compile-time reflection endpoints.

## 4. Risks (Memory, AI-legibility, Parse ambiguity)
- **Tooling Breaks / Infinite Recursion:** Macro expansion could crash the LSP or create cyclic infinite loops in the compiler. 
- **Mitigation:** Enforce strict recursion limits in the expansion engine. Implement hygienic AST-based macros only, guaranteeing that LSPs and AI agents can query the fully *expanded* AST structure safely.

## 5. Acceptance Criteria
- Code with macro definitions and macro calls parses successfully.
- Macros expand correctly during the semantic phase, properly replacing identifiers with correct hygiene.
- Attempting cyclic macro expansion fails cleanly with an error (no compiler crashes).
- LSPs and AI diagnostics can inspect the fully expanded AST structure natively.
- Tests cover macro hygiene, recursion depths, and typical boilerplate generation.
- Formatter supports the new `macro` block syntax.
