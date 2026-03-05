# M38-S64 Security-Headers X-Frame-Options Invalid Env Fallback

## What it is

M38-S64 adds runtime e2e coverage that invalid `SEC4_RT_SECURITY_HEADERS_X_FRAME_OPTIONS` env values deterministically fall back to `DENY`.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Runtime already clamps x-frame-options values to `DENY`/`SAMEORIGIN`, but this fallback behavior was not explicitly locked by an end-to-end contract test.

This slice ensures malformed env values cannot alter response headers unpredictably.

## How it works

1. Added clang-gated HTTP runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_x_frame_options_invalid_env_falls_back_to_deny_when_enabled`
2. The test builds a real c-bin service with `sec.withSecurityHeaders(...)` and starts it in oneshot mode.
3. It runs the service with:
   - `SEC4_RT_SECURITY_HEADERS_X_FRAME_OPTIONS=ALLOW-FROM`
4. The test asserts deterministic fallback output:
   - response contains `X-Frame-Options: DENY`
   - response does not contain the invalid env token.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_x_frame_options_invalid_env_falls_back_to_deny_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds a focused e2e contract test and slight runtime-test suite overhead.
- Improves confidence that security-header env hardening remains deterministic across future runtime refactors.

## Next

1. Add referrer-policy invalid-env fallback coverage (`M38-S65`) so malformed values deterministically clamp to `strict-origin-when-cross-origin`.
2. Continue converting security-header fallback assumptions into explicit runtime contracts.
