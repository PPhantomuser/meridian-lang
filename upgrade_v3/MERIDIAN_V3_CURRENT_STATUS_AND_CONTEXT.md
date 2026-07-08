# Meridian V3 — Current Status and Context

## Current known state

Meridian has evolved through two stages so far:

### V1
Meridian v1 established the minimal working language/compiler pipeline:
- lexer,
- parser,
- AST,
- semantic/type checking,
- Cranelift-backed execution path,
- minimal CLI/tooling.

The original vision positioned Meridian as a serious, human-and-AI-friendly language rather than a toy experiment.

### Current guide state
From the current language guide, Meridian already shows support for:
- primitive types: `Number`, `String`, `Bool`, `Unit` / `()`,
- `let` and `let mut`,
- local type inference,
- arithmetic and relational operators,
- grouped expressions with `()` support,
- `if / else`,
- `while`,
- range-based `for`,
- `break` / `continue`,
- functions with implicit return,
- comments,
- imports,
- `print`,
- CLI workflows such as `merid run` and `merid fmt`.

### V2 design state
The V2 upgrade spec defines Meridian v2 as the serious usability core and adds or plans:
- mutation + control flow maturity,
- structs, enums, tuples, vectors/maps/sets,
- pattern matching,
- generics and traits,
- `Option<T>` and `Result<T, E>`,
- ARC for heap-managed data,
- references and limited aliasing rules,
- modules/packages,
- stdlib expansion,
- basic concurrency,
- interop/deployment foundations,
- tooling and AI-legibility improvements.

The same spec explicitly pushes several advanced areas to **v3+**, such as:
- full async/await,
- cross-function lifetime/borrowing sophistication,
- richer ownership semantics,
- macros/metaprogramming,
- hosted package registry,
- advanced tooling maturity,
- native ML strategy.

## What V3 is therefore responsible for

V3 should be treated as the **advanced core and runtime upgrade**, not as a total rewrite.

It must build on:
- the original vision,
- the current implemented language surface,
- the v2 capability roadmap.

It should not assume Meridian starts from zero.

## Core caution for V3

The following topics are architecture-sensitive and must be specified carefully before implementation:
- async/await and its runtime model,
- borrowing across function boundaries and suspension points,
- macro system design,
- registry/package trust model,
- machine-readable schemas for AI tooling.
