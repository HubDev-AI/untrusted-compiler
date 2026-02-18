# M39: LASM Overflow Parser Hardening

## Why

When LASM non-oneshot mode hit queue saturation (`max_concurrency` + `max_pending`), overflow connections always returned generic `503` busy envelopes.

That hid malformed-request diagnostics on overload paths and let invalid HTTP inputs bypass the parser hardening already enforced on normal worker paths.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added shared request-head parser `read_lasm_http_request_head(...)` and moved request-line/header validation there.
2. Updated `read_lasm_http_request(...)` to reuse the shared head parser and only perform body-limit + body-read logic afterward.
3. Updated overload (`TrySendError::Full`) handling to:
   - parse/validate overflow request heads,
   - return parser-status JSON envelopes (for malformed requests),
   - preserve `503` busy fallback for probe/no-data timeout-style overflow connects.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_path_returns_parse_error_for_malformed_request`
   - verifies malformed overflow request returns deterministic parser envelope (`505` / `HTTP.VERSION_NOT_SUPPORTED`).
2. `cargo test -p sec4 --test commands run_command_lasm_backend_returns_503_when_concurrency_limit_is_reached`
   - verifies busy fallback behavior still returns deterministic `503`.
