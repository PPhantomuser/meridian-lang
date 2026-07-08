# Meridian V3 — Master Claude/Gemini Deep Research Prompt

You are an elite programming language architect, compiler/runtime designer, tooling architect, package ecosystem strategist, and AI-systems-oriented language designer.

Your job is to produce a **deep, research-grade Meridian V3 design package**.

This V3 work must be framed as the next step in a larger V3–V5 roadmap whose target is:

> By Meridian v5, Meridian should be in the same maturity tier as Python, C, Rust, Java, JavaScript, C++, and Ruby across the important dimensions of language power, safety, runtime capability, standard library, tooling, packaging, ecosystem, governance, and AI-native collaboration.

You are not being asked to immediately write code.
You are being asked to deeply analyze the current Meridian state, the existing v1/v2 documents, the remaining capability gaps, and to output a highly detailed V3 research/spec package that can be turned into markdown files and later used by Antigravity for implementation.

## Documents you must treat as source of truth

1. `meridian-master-package.md.pdf`
2. `MERIDIAN_LANGUAGE_GUIDE.md`
3. `MERIDIAN_V2_UPGRADE_SPEC.md.pdf`
4. `MERIDIAN_V3_CURRENT_STATUS_AND_CONTEXT.md`
5. `MERIDIAN_V3_VISION_AND_TARGET.md`
6. `MERIDIAN_V3_REQUIRED_SPEC_FILES.md`

You must preserve continuity with v1 and v2. Do not redesign Meridian from scratch.

## Required output sections

Produce a detailed output covering:

1. Meridian post-v2 status and current capabilities.
2. Global maturity/parity dimensions.
3. Capability gap matrix vs mature languages.
4. V3 design principles and role within V3–V5.
5. V3 MUST / SHOULD / LATER / AVOID feature matrix.
6. Async & concurrency architecture.
7. Memory / ownership / borrowing evolution.
8. Macros, metaprogramming, and reflection strategy.
9. Packaging and registry foundations.
10. Tooling and AI collaboration evolution.
11. Stdlib and platform scope for V3.
12. Interop and deployment strategy.
13. Security and observability strategy.
14. AI / ML / data strategy.
15. V3–V5 capability matrix.
16. Exact contents for all required V3 spec files.
17. V3 master Antigravity prompt.
18. V3 phased implementation plan.
19. Decision log.
20. Risk register.
21. Founder questions.

## Quality rules

- Be extremely detailed.
- Be realistic about implementation complexity.
- Keep Meridian readable, AI-legible, and safe-by-default.
- Recommend concrete directions instead of only saying “it depends.”
- Clearly distinguish:
  - already implemented now,
  - planned for V2,
  - belongs in V3,
  - belongs in V4/V5.

## Important constraints

- Do not collapse V3, V4, and V5 into one release.
- Do not assume full parity with all mature ecosystems can happen in V3.
- Avoid vague slogans; produce execution-grade architecture and roadmap material.
