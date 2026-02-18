# M39 - LASM Smoke Runtime Script and Timeout Flag

## What Was Added

Extended `sec4 lasm-smoke` with deterministic runtime action scripting and runtime timeout configuration so timeout/abort behavior can be exercised from CLI without changing source programs.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `sec4 lasm-smoke` now accepts:
   - `--runtime-script <csv>`
   - `--max-request-ms <N>`
2. Runtime script segments support:
   - `yield`
   - `sleep:<ms>`
   - `complete:<code>`
3. Invalid runtime-script segments fail deterministically with usage-style exit code (`2`) and clear segment-level guidance.
4. Empty runtime-script payloads (no actionable segments after trimming) fail deterministically.
5. `--max-request-ms 0` is rejected deterministically (`must be >= 1ms`).
6. JSON/text summaries now include effective `maxRequestMs` value.
7. Timeout scenarios driven by scripted sleeps emit deterministic timeout responses (`504`, `handler timed out after <limit>ms`) and are reflected in `statusCounts`/`errorCount`.

## Why

This gives a deterministic CLI probe surface for LASM runtime timeout behavior, making async timeout/cancellation paths easy to validate from smoke command runs without introducing extra fixture programs.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_`

New coverage:

- `lasm_smoke_command_rejects_zero_max_request_ms`
- `lasm_smoke_command_rejects_invalid_runtime_script_segments`
- `lasm_smoke_command_reports_timeout_status_with_max_request_ms`
