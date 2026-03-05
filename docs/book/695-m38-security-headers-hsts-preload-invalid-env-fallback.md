# M38-S72 Security-Headers HSTS Preload Invalid Env Fallback

## What it is

M38-S72 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD` env values deterministically keep default preload-disabled behavior when HSTS is enabled.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The HSTS preload toggle is parsed through strict env-boolean logic and defaults to disabled. This fallback behavior needed explicit e2e contract coverage so malformed env tokens cannot silently add `preload` to emitted HSTS headers.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_hsts_preload_invalid_env_falls_back_to_disabled_when_hsts_enabled`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED=1`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS=31536000`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS=1`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD=MAYBE` (invalid)
3. Test asserts deterministic fallback behavior:
   - response contains `Strict-Transport-Security: max-age=31536000; includeSubDomains`
   - response does not contain `; preload`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_preload_invalid_env_falls_back_to_disabled_when_hsts_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused fallback e2e test and slight runtime-suite overhead.
- Improves confidence that malformed env values cannot incorrectly add HSTS preload semantics.

## Next

1. Add CSP policy invalid-env fallback coverage (`M38-S73`) to lock deterministic fallback to default CSP policy when env header value is invalid.
2. Continue security-header fallback hardening until each env-driven value path has explicit runtime contract coverage.
