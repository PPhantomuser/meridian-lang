# Meridian V3 — Required Spec Files

Before Antigravity begins serious V3 implementation, Claude/Gemini should generate and refine the following spec files.

## Required V3 spec pack

1. `MERIDIAN_V3_VISION_AND_PRINCIPLES.md`
   - V3 identity, design philosophy, relationship to V4/V5.

2. `MERIDIAN_V3_CAPABILITY_GAP_MATRIX.md`
   - Gap analysis vs mature languages and mapping of missing areas to V3/V4/V5.

3. `MERIDIAN_V3_ASYNC_AND_CONCURRENCY_SPEC.md`
   - Async syntax, runtime model, task model, cancellation, structured concurrency, threads integration.

4. `MERIDIAN_V3_MEMORY_MODEL_SPEC.md`
   - Ownership/borrowing/lifetimes evolution beyond V2.

5. `MERIDIAN_V3_MACROS_AND_METAPROGRAMMING_SPEC.md`
   - Declarative macros, possible procedural macro strategy, reflection boundaries, tooling implications.

6. `MERIDIAN_V3_PACKAGING_AND_REGISTRY_FOUNDATIONS.md`
   - Manifest, lockfile, workspaces, dependency resolution, registry model, package trust/provenance.

7. `MERIDIAN_V3_TOOLING_AND_AI_GUIDE.md`
   - LSP goals, JSON AST/IR/diagnostic schemas, AI metadata, formatter/linter/test/docgen expectations.

8. `MERIDIAN_V3_STDLIB_AND_PLATFORM_CORE.md`
   - Which stdlib/platform features must exist in V3 and which belong to V4.

9. `MERIDIAN_V3_INTEROP_AND_DEPLOYMENT_SPEC.md`
   - C FFI, AOT targets, Python bridge direction, WASM direction, deployment goals.

10. `MERIDIAN_V3_SECURITY_AND_OBSERVABILITY_SPEC.md`
   - Safe/unsafe boundaries, supply chain, tracing/logging, reproducibility, auditability.

11. `MERIDIAN_V3_AI_ML_DATA_STRATEGY.md`
   - Interop-first vs native-first AI strategy for V3 and beyond.

12. `MERIDIAN_V3_IMPLEMENTATION_PLAN.md`
   - Phase-by-phase implementation roadmap for V3.

13. `MERIDIAN_V3_DECISION_LOG.md`
   - Major design decisions, alternatives, reasons, revisit points.

14. `MERIDIAN_V3_RISK_REGISTER.md`
   - Major risks, probability, impact, mitigation, phase.

15. `MERIDIAN_V3_ACCEPTANCE_TEST_STRATEGY.md`
   - Golden tests, schema tests, semantic tests, async/runtime tests, tooling acceptance tests.

## Operating rule

Antigravity should not invent these specs while coding V3 ad hoc.

These files should exist first, be reviewed by the human founder, and then become the execution source of truth.
