# Untrusted<T> Agent Collaboration Guide

This file defines the minimum working agreement for any AI agent contributing to Untrusted<T>.

## Project Root

- `.`

## Source of Truth

- Master roadmap: `docs/05-sec4-master-roadmap.md`
- Book index: `docs/book/README.md`
- Security baseline: `docs/book/53-v0-security-baseline.md`
- Security stdlib/API contract: `docs/book/54-v0-stdlib-security-surface.md`
- Security typing/effects rules: `docs/book/55-v0-typing-effects-security-rules.md`

## Development Mode

- Build in milestone order (`M0` -> `M8`), one milestone-sized slice at a time.
- Prefer the smallest end-to-end, verifiable increment.
- Keep v0.1-lite constraints: no inheritance/trait complexity, no unnecessary language features.
- Follow security-first constraints: capabilities + effects + policy + typed sinks + explicit trust gates.

## Required Workflow Per Milestone

1. Read current roadmap status and milestone scope.
2. Implement the milestone slice.
3. Run verification commands relevant to changed areas.
4. Update docs as book chapters (not ad-hoc notes).
5. Explain each implemented part with:
   - what it is
   - why it exists
   - how it works
   - inputs/outputs and constraints
   - failure modes/diagnostics
   - example usage
   - tradeoffs/next steps
6. Create a git commit for that milestone (`one commit per M`).

## Commit Policy

- Mandatory: create at least one commit after each milestone completion.
- Commit message format (recommended): `M<N>: <short milestone summary>`.
- Do not rewrite or amend old milestone commits unless explicitly requested.

## Verification Baseline

From repo root, run as needed:

- `cargo test -q`
- `cargo run -q -p sec4 -- check --path examples/hello`
- `cargo run -q -p sec4 -- check --path examples/hello --emit ast`
- `cargo run -q -p sec4 -- build --path examples/hello`

## Guardrails

- Do not ship code-only changes without docs updates.
- Do not skip verification before claiming milestone completion.
- Preserve deterministic diagnostics with stable error codes.
- Keep changes explicit and auditable over clever shortcuts.
