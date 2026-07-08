# Meridian V3 Pack — Start Here

This pack is the starter operating set for **Meridian V3**.

It is designed to help Antigravity continue from the current Meridian state and the existing Meridian documents, while keeping the long-term **V5 parity goal** in view.

## What this pack is for

This pack is **not** the full final V3 specification. It is the **execution starter pack** that tells Claude/Antigravity:

- what Meridian currently is,
- what documents are the current source of truth,
- what V3 is supposed to accomplish,
- what must be researched/spec'd before coding,
- how to phase the work safely.

## Source-of-truth documents Antigravity must read with this pack

These should be present in the workspace together with this V3 pack:

1. `meridian-master-package.md.pdf`
2. `MERIDIAN_LANGUAGE_GUIDE.md`
3. `MERIDIAN_V2_UPGRADE_SPEC.md.pdf`
4. This V3 pack

## Files in this pack

1. `MERIDIAN_V3_CURRENT_STATUS_AND_CONTEXT.md`
   - Current Meridian state and what is already known.
2. `MERIDIAN_V3_VISION_AND_TARGET.md`
   - V3 purpose and V5 parity target.
3. `MERIDIAN_V3_REQUIRED_SPEC_FILES.md`
   - The exact V3 spec files Claude should generate before implementation.
4. `MERIDIAN_V3_MASTER_CLAUDE_PROMPT.md`
   - Main Claude/Gemini Deep Research prompt for V3.
5. `MERIDIAN_V3_MASTER_ANTIGRAVITY_PROMPT.md`
   - Main execution prompt for Antigravity after the V3 spec files exist.
6. `MERIDIAN_V3_PHASE_GUIDE.md`
   - Suggested V3 phase breakdown.
7. `MERIDIAN_V3_WORKFLOW_INSTRUCTIONS.md`
   - How to use this pack with Claude and Antigravity.

## Important operating rule

Do **not** jump straight into coding all of V3.

Correct order:
1. Read current docs.
2. Use the Claude V3 prompt to generate the missing V3 spec files.
3. Review the specs manually.
4. Feed the reviewed spec files + Antigravity master prompt into Antigravity.
5. Execute V3 phase by phase.

## Why this matters

V3 contains high-risk language-design areas:
- async/await,
- borrowing/lifetimes,
- macros/metaprogramming,
- package/registry foundations,
- AI-legible schemas/tooling.

These must be specified carefully before implementation to avoid architecture drift.
