# M38-S63 Security-Headers HSTS Invalid Env Fallback

## What it is

M38-S63 adds direct runtime e2e coverage proving invalid or negative `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS` values fall back to a deterministic safe default.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S61 implemented HSTS materialization and valid env controls, but fallback behavior for malformed numeric env values was only implicit in code.

This slice locks that behavior as a contract:

- invalid max-age input must not leak into emitted headers
- negative max-age input must not leak into emitted headers
- runtime must continue emitting deterministic safe HSTS output

## How it works

1. Added a clang-gated HTTP runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_hsts_invalid_max_age_falls_back_to_default_when_enabled`
2. The test builds a real c-bin service with `sec.withSecurityHeaders(...)` and runs it in oneshot mode.
3. It runs two env-driven cases:
   - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS=not-a-number`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS=-7`
4. Each case asserts deterministic fallback header materialization:
   - `Strict-Transport-Security: max-age=15552000; includeSubDomains`
   - no reflection of invalid/negative raw input in output header values.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_invalid_max_age_falls_back_to_default_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused runtime contract test, increasing test runtime slightly.
- Keeps fallback guarantees explicit and regression-resistant without expanding runtime policy surface.

## Next

1. Add `x-frame-options` invalid-env fallback coverage (`M38-S64`) so malformed env values deterministically clamp to `DENY` in runtime header output.
2. Continue security-header hardening slices that convert implicit fallback logic into explicit e2e contracts.
