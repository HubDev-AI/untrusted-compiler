# M39 - LASM Smoke Response Header Extraction

## What Was Added

Extended `sec4 lasm-smoke` response-plan extraction with deterministic header support.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `lasm-smoke` now extracts static response headers from reachable handler call graphs:
   - `res.setHeader(name, value)`
2. Header extraction supports typed gate patterns and aliases:
   - direct wrappers `headers.name("...")` / `headers.value("...")`
   - local let bindings that alias those wrapper outputs
3. Extracted headers are carried into runtime response plan registration and surfaced in output:
   - text mode: `headerCount=<n>`
   - json mode: `headers` object

## Why

Status/body-only smoke summaries under-report real handler behavior. Header extraction improves runtime signal quality for integration and policy-facing routes.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_`
2. `cargo test -p sec4 --test commands`

Updated coverage:

- `lasm_smoke_command_emits_json_summary_when_requested`
