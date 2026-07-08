# Meridian V3 — Workflow Instructions

## Recommended workflow

### Step 1 — Prepare workspace
Make sure these files are present:
- `meridian-master-package.md.pdf`
- `MERIDIAN_LANGUAGE_GUIDE.md`
- `MERIDIAN_V2_UPGRADE_SPEC.md.pdf`
- this V3 pack
- the Meridian source tree

### Step 2 — Use Claude/Gemini first
Do **not** start V3 implementation directly.

First, use:
- `MERIDIAN_V3_MASTER_CLAUDE_PROMPT.md`

Ask Claude/Gemini to generate the missing V3 spec files listed in:
- `MERIDIAN_V3_REQUIRED_SPEC_FILES.md`

### Step 3 — Human review
Review the generated V3 spec files manually.
Pay special attention to:
- async/await model,
- borrowing/lifetime commitments,
- macro/reflection scope,
- packaging/registry assumptions,
- AI tooling schema changes.

### Step 4 — Hand reviewed specs to Antigravity
Once the V3 spec files are reviewed, give Antigravity:
- the source-of-truth docs,
- the reviewed V3 spec files,
- `MERIDIAN_V3_MASTER_ANTIGRAVITY_PROMPT.md`

### Step 5 — Execute phase by phase
For each V3 phase:
1. Antigravity writes the planning file.
2. You review the planning file.
3. Antigravity implements only that phase.
4. You review code/tests/docs.
5. Only then move to the next phase.

## Do not do this

- Do not ask Antigravity to “finish V3 in one go.”
- Do not let it invent missing semantics silently.
- Do not skip the V3 spec-generation step.
- Do not merge V3/V4/V5 into one implementation mission.

## Practical goal

This workflow keeps Meridian coherent, ambitious, and actually buildable.
