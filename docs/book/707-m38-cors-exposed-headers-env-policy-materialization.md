# M38-S84 CORS Exposed-Headers Env Policy Materialization

## What it is

M38-S84 adds runtime materialization for `SEC4_RT_CORS_EXPOSED_HEADERS` and deterministic fallback behavior for malformed values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CORS exposed-headers is part of the policy surface, but runtime success responses did not emit `Access-Control-Expose-Headers`. This prevented policy-driven exposure behavior and left a parity gap between policy intent and runtime output.

## How it works

1. Extended runtime CORS policy/router state with `expose_headers`.
2. Added env loading for `SEC4_RT_CORS_EXPOSED_HEADERS`.
3. Added deterministic validation:
   - only valid header-name CSV tokens are accepted
   - invalid lists clamp to empty (no exposed-headers header).
4. Updated runtime CORS success header block:
   - emits `Access-Control-Expose-Headers: ...` when configured.
5. Added clang-gated e2e coverage:
   - valid exposed-headers list emission
   - invalid list fallback to absent header.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_exposed_headers_from_env_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_exposed_headers_invalid_env_fall_back_to_absent_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one runtime policy field and header-render path for CORS success responses.
- Slightly increases e2e test count, but removes a real policy-materialization gap and keeps env behavior deterministic.

## Next

1. Continue middleware policy materialization hardening for remaining env-backed security fields.
2. Keep runtime hardening changes paired with focused e2e coverage and roadmap/book traceability.
