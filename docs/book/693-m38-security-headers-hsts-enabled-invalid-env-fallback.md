# M38-S70 Security-Headers HSTS-Enabled Invalid Env Fallback

## What it is

M38-S70 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED` env values deterministically keep default HSTS-disabled behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

HSTS enablement is controlled by strict env-boolean parsing with default `false`. Fallback behavior for malformed tokens needed explicit e2e contract coverage so invalid env inputs cannot silently enable HSTS.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_hsts_enabled_invalid_env_falls_back_to_disabled_by_default`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED=MAYBE` (invalid)
   - valid `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS` to ensure only enable flag controls behavior.
3. Test asserts deterministic fallback behavior:
   - response does not include `Strict-Transport-Security` header.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_enabled_invalid_env_falls_back_to_disabled_by_default`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused e2e fallback test and minor suite runtime cost.
- Improves confidence that malformed env values cannot unexpectedly strengthen/deviate HSTS behavior from policy defaults.

## Next

1. Add HSTS includeSubDomains invalid-env fallback coverage (`M38-S71`) to lock boolean fallback semantics for that sub-control when HSTS is enabled.
2. Continue security-header fallback hardening until all HSTS sub-flags have explicit deterministic coverage.
