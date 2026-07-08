# Tooling and LSP Roadmap

## Required official tools

- `merid new`
- `merid run`
- `merid build`
- `merid test`
- `merid fmt`
- `merid doc`
- `merid ast`

## Editor and IDE direction

Meridian should support a real LSP implementation using shared compiler crates.
The LSP should not be a drifting reimplementation.

## Early tooling goals

- diagnostics,
- formatting,
- AST export,
- test running,
- simple docs generation.

## Later tooling goals

- go-to-definition,
- hover,
- completion,
- structural refactors,
- codemods,
- machine-consumable analysis outputs.
