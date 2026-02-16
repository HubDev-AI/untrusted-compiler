# M38-S66 Security-Headers X-Content-Type-Options Invalid Env Fallback

## What it is

M38-S66 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_X_CONTENT_TYPE_OPTIONS` env values deterministically fall back to enabled `nosniff` behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`x-content-type-options` is configured through strict env boolean parsing, but fallback semantics were not explicitly locked by a dedicated runtime contract test.

This slice ensures malformed boolean tokens cannot silently disable `nosniff`.

## How it works

1. Added clang-gated HTTP runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_x_content_type_options_invalid_env_falls_back_to_nosniff_when_enabled`
2. The harness builds a c-bin service with `sec.withSecurityHeaders(...)` and runs oneshot mode.
3. Service is started with invalid boolean env token:
   - `SEC4_RT_SECURITY_HEADERS_X_CONTENT_TYPE_OPTIONS=MAYBE`
4. Test asserts deterministic fallback output:
   - response contains `X-Content-Type-Options: nosniff`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_x_content_type_options_invalid_env_falls_back_to_nosniff_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused e2e contract test and small runtime-test suite overhead.
- Improves hardening confidence without changing runtime header behavior itself.

## Next

1. Add CSP report-only invalid-env fallback coverage (`M38-S67`) to lock enforce/report-only defaulting behavior under malformed env inputs.
2. Continue security-header fallback slices until each env-driven branch has explicit deterministic runtime coverage.
