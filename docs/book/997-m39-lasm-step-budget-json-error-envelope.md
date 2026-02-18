# M39: LASM Step-Budget JSON Error Envelope

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM runtime step-budget exhaustion responses now emit JSON error envelopes instead of plain-text-only bodies.
- Deterministic envelope fields:
  - `code`: `LASM.STEP_BUDGET_EXCEEDED`
  - `kind`: `internal`
  - `message`: `run failed: LASM runtime remained active after step budget (<N>)`
  - trace/time metadata remains aligned with existing LASM error envelopes.

## Why

Step-budget failures are contract-level runtime errors and should match the same structured error envelope behavior used by other deterministic LASM failure paths.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_enforces_max_runtime_steps_from_policy`
- `cargo test -p sec4 --test commands run_command_lasm_backend_counts_runtime_step_failures_toward_keep_alive_limit`
