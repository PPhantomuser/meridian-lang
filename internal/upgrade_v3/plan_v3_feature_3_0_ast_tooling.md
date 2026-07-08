# Mission V3.0 Scope Summary: AST & Tooling Foundation

## 1. Scope & Goals
- Stabilize the AST architecture for V3, making it strictly machine-readable.
- Implement the `--ai-diagnostics` flag to output compiler diagnostics in JSON format.
- Set up the basic Language Server Protocol (LSP) skeleton (`merid-lsp`).
- Establish stable JSON schemas for AST/IR/diagnostics serialization.

## 2. Out-of-Scope Items
- **NO semantic changes to the language.** Do not touch `async`, borrowing/lifetimes, or macros.
- Do not implement new syntax (e.g., `async fn`, `&mut`, `macro_rules`).
- Do not build advanced LSP features like auto-refactoring; only bootstrap the server skeleton (startup/shutdown, initialize handshake).

## 3. Affected Compiler Crates/Modules
- `ast/`: Refine AST node structures to ensure robust serialization/versioning.
- `parser/`: Parsing logic updates strictly for emitting stable ASTs (no new grammar).
- `diagnostics/`: Add JSON generation pipelines alongside the standard text output.
- `cli/`: Wire up the `--ai-diagnostics` flag and add a command to start the LSP.
- `lsp/` (New Crate): Implement the core Language Server Protocol loop and handshake.

## 4. Risks (Memory, AI-legibility, Parse ambiguity)
- **Parse Ambiguity / Refactoring:** Modifying the AST or parser risks introducing regressions in existing V1/V2 syntax parsing.
- **AI-legibility:** The diagnostics schema might become too complex or brittle for AI tools to parse reliably.
- **Mitigation:** Rely on strict versioned schemas for JSON outputs, keep AST representations explicitly mapped, and ensure backward compatibility for tooling. Add regression tests for all V2 constructs.

## 5. Acceptance Criteria
- Existing code compiles exactly as before without semantic changes.
- `merid run --ai-diagnostics <file>` outputs fully machine-readable JSON containing structured errors (e.g., `code`, `file`, `context`).
- The `merid-lsp` binary can boot, successfully establish the `initialize` handshake with an editor, and shut down cleanly.
- Unit and integration tests verify the JSON output format and LSP initialization.
- Updates provided to `merid fmt` and the AST schema documentation (if any internal AST nodes changed shape).
