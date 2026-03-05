# M39 - LASM Request-Target Normalization

## Summary

Extended LASM request-line parsing to normalize absolute-form request targets into route-matchable paths and reject invalid target forms deterministically.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now accepts:
     - origin-form targets (for example `/health`),
     - absolute-form targets (for example `http://localhost/health`),
     - asterisk-form target (`*`).
   - Absolute-form targets are normalized to path-only form before route matching.
   - Invalid targets are rejected deterministically with `400 Bad Request` and `invalid request target` diagnostics.
   - Existing query stripping behavior remains intact on normalized targets.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_accepts_absolute_form_request_target`.
   - Added `run_command_oneshot_lasm_backend_returns_400_for_invalid_request_target`.

## Why

Absolute-form targets appear in proxy-style clients; without normalization they fail route lookup despite valid paths. Deterministic normalization improves compatibility while preserving strict parser behavior for malformed target forms.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_request_target`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_missing_host_header`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
