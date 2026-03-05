# M39 - LASM Smoke Duration Metrics

## What Was Added

Added request timing metadata to LASM HTTP exchanges and surfaced deterministic duration metrics in `sec4 lasm-smoke` summaries.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`
2. `compiler/sec4-cli/src/main.rs`
3. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `HttpExchange` now carries:
   - `request_started_at_ms`
   - `response_ready_at_ms`
   - `duration_ms()` helper
2. LASM runtime now stamps timing metadata for:
   - normal completions,
   - timeout responses,
   - queue overflow responses,
   - route-miss responses.
3. `sec4 lasm-smoke` summaries now include deterministic duration metrics:
   - `durationMs.min`
   - `durationMs.max`
   - `durationMs.avg`
   - `firstDurationMs`
4. Text summary also reports:
   - `durationMinMs`
   - `durationMaxMs`
   - `durationAvgMs`

## Why

This gives first-class timing signal for LASM async runtime behavior directly in smoke output, allowing deterministic latency checks alongside status/error envelopes.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`

New coverage:

- `lasm_smoke_command_times_out_pending_requests_from_submit_age`
- `lasm_smoke_command_reports_duration_stats_in_json_summary`
