# M39 - LASM Overload JSON Error Envelope

## Summary

Converted LASM non-oneshot overload (`worker queue full`) responses from plain text to deterministic JSON error envelopes with trace metadata.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - In `cmd_run_lasm_backend(...)` overflow path (`TrySendError::Full`):
     - response now uses JSON error envelope with:
       - code: `HTTP.SERVICE_UNAVAILABLE`
       - kind: `internal`
       - message: `server busy: max concurrency reached`
       - status: `503`
       - `traceId` and `timeMs`
   - Reused a request-scoped trace id for header/body parity in this path.
   - Removed unused `stamp_lasm_trace_id(...)` helper after trace-id refactor.

## Why

This keeps LASM overload behavior consistent with the benchmark error-envelope direction and improves machine-readability for overload diagnostics while preserving deterministic status/message semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_returns_503_when_concurrency_limit_is_reached`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_omits_cors_headers_for_disallowed_origin`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_honors_max_pending_override_before_overflow`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
