# M39 - LASM Smoke JSON Summary Output

## What Was Added

Extended `sec4 lasm-smoke` output contracts and response extraction coverage.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `sec4 lasm-smoke` now supports `--format text|json` (default: `text`).
2. JSON mode emits a deterministic machine-readable summary payload with:
   - `ok`
   - `requestId`
   - `responseRequestId`
   - `entry`
   - `origin`
   - `requests`
   - `okCount`
   - `errorCount`
   - `steps`
   - `nowMs`
   - `status`
   - `body`
3. Response plan extraction used by `lasm-smoke` now recognizes:
   - `res.text`
   - `res.html`
   - `res.json`
   - `res.ok`
   - `res.okMeta`
4. Helper-call-graph traversal remains active for both route registration and response discovery.

## Why

`lasm-smoke` is now usable both for human debugging (`text`) and automation/CI contract checks (`json`) while staying implementation-first and deterministic.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_`
2. `cargo test -p sec4 --test commands`

New coverage:

- `lasm_smoke_command_extracts_res_ok_status_and_body`
- `lasm_smoke_command_emits_json_summary_when_requested`
