# M39: LASM Parser Error CORS Suppression

## Why

After parser errors were moved to structured JSON envelopes, malformed requests could still receive default CORS headers from the response writer path.

For malformed/invalid HTTP input, emitting permissive default CORS headers is unnecessary and weakens failure-path boundary clarity.

## What Changed

In `compiler/sec4-cli/src/main.rs` (`process_lasm_connection_with_runtime`):

1. Parser-failure response emission now sets `include_cors_defaults = false` when calling `write_lasm_http_response(...)`.
2. This suppresses default CORS headers on parser-failure paths while preserving:
   - structured JSON error envelope body,
   - deterministic status and error message,
   - `X-Trace-Id` metadata.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_method_token`
   - now asserts parser failure does **not** include `Access-Control-Allow-Origin`.
