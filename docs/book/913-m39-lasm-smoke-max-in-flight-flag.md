# M39 - LASM Smoke Max In-Flight Flag

## What Was Added

Extended CLI `lasm-smoke` with optional runtime concurrency-gate wiring.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `sec4 lasm-smoke` now accepts:
   - `--max-in-flight <N>`
2. When provided, CLI applies the limit to LASM HTTP runtime before request submission.
3. `--max-in-flight 0` fails deterministically with exit code `2` and validation guidance.
4. Text/JSON smoke summaries now include effective max-in-flight visibility (`maxInFlight`) so concurrency-gated runs are observable.
5. Step-budget exhaustion failure now reports active queue state (`in_flight`, `pending`) for deterministic async-path debugging.

## Why

This exposes the new LASM in-memory concurrency gate through the smoke command so async-flow-control behavior can be exercised from CLI without adding separate harness tooling.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_rejects_zero_max_in_flight`
2. `cargo test -p sec4 --test commands lasm_smoke_command_reports_effective_max_in_flight_in_text_summary`
