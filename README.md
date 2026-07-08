# Meridian Antigravity Delivery Pack

This pack is the starter package to give Antigravity along with the Claude-generated Meridian master package.

## What to provide to Antigravity

Give Antigravity these items together:

- The original Claude master package PDF / source document.
- All files inside `specs/` from this pack.
- The `ANTIGRAVITY_FIRST_MISSION.md` file as the first mission prompt.
- The `ANTIGRAVITY_USAGE_INSTRUCTIONS.md` file as the operating instructions.

## Recommended workflow

1. Do **not** ask Antigravity to build the whole language in one go.
2. First make Antigravity read all spec files and summarize v1 scope.
3. Then run only the first mission.
4. After each mission, review output manually.
5. Keep the spec files as the source of truth.
6. Update specs before starting the next large phase.

## Mission order

1. `ANTIGRAVITY_FIRST_MISSION.md`
2. Parser + AST hardening mission
3. Type checker mission
4. MIR + VM mission
5. Tooling mission
6. LSP / diagnostics mission
7. Cranelift backend mission

## Important principle

Meridian is a new language built from scratch in design and semantics, but its compiler/toolchain may be implemented in Rust first. That is expected and correct.
