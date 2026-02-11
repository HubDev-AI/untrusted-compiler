---
name: ailang-dev-workflow
description: "Execute AILang implementation work from planning through code, tests, and documentation. Use when building or changing compiler/runtime/stdlib features, refining language rules, adding security constraints, or preparing releases. Enforce a strict workflow: pick milestone scope, implement smallest vertical slice, validate with tests, update docs as book chapters, and provide a clear explanation for every implemented part (what it is, how it works, why chosen, tradeoffs)."
---

# AILang Dev Workflow

## Overview
Use this skill to keep AILang development disciplined and cumulative: implementation, testing, and documentation move together so `docs/` evolves into a coherent book, not scattered notes.

## Workflow
1. Read current scope from `docs/05-ailang-master-roadmap.md`.
2. Select one milestone-sized task or smaller vertical slice.
3. Implement only the smallest end-to-end change that can be tested.
4. Run relevant verification (`build/check/test/lint` for touched area).
5. Update docs in `docs/` with:
- Technical implementation detail.
- Reader-facing explanation using the required template.
6. Create a git commit after each milestone completion (`one commit per M`).
7. Report outcomes with:
- What changed.
- Why this approach.
- How it works.
- What remains.

## Non-Negotiable Rules
- Never finish a feature with code only; docs update is required.
- Never finish a feature without at least one verification step.
- Never close a milestone without creating a commit for that milestone.
- Keep v0.1-lite constraints: no trait/inheritance webs, no unnecessary complexity.
- Prefer explicit, auditable behavior over convenience magic.

## Required Explanation Format
For every implemented part, include all sections:
1. What it is.
2. Why it exists.
3. How it works internally.
4. Inputs/outputs and constraints.
5. Failure modes and diagnostics.
6. Example usage.
7. Tradeoffs and next steps.

Use the template in `references/explanation-template.md`.

## Documentation Book Mode
When adding or changing features, write docs so the set reads as a progressive book:
- Concept and motivation.
- Formal behavior/spec.
- Implementation details.
- Worked example.
- Troubleshooting.

Use `references/book-writing-checklist.md` before closing work.

## Scope Priorities
When uncertain, prioritize in this order:
1. Parser/type/effects/MIR correctness.
2. Backend/runtime reliability.
3. Security-by-construction guarantees.
4. Documentation clarity and explainability.

## Resources
- Workflow guide: `references/workflow-checklist.md`
- Explanation template: `references/explanation-template.md`
- Book writing checklist: `references/book-writing-checklist.md`
- Helper script for explanation stubs: `scripts/new_explanation_entry.py`

## Usage Notes
- Use this skill for both planning and implementation turns.
- Keep changes incremental and verifiable.
- If a request is broad, split into milestones and deliver one verifiable slice at a time.
