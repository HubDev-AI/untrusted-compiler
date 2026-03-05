# M38-S69 Security-Headers Enabled Invalid Env Fallback

## What it is

M38-S69 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_ENABLED` env values deterministically keep security headers enabled.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The top-level security-headers enabled flag is parsed through strict env-boolean logic with default `enabled=true` behavior. This fallback behavior needed explicit e2e contract coverage so malformed env tokens cannot silently disable baseline header posture.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_enabled_invalid_env_falls_back_to_enabled_when_enabled_by_default`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_ENABLED=MAYBE` (invalid token)
3. Test asserts deterministic fallback behavior:
   - response contains baseline security header `X-Content-Type-Options: nosniff`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_enabled_invalid_env_falls_back_to_enabled_when_enabled_by_default`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused e2e fallback test and slight suite runtime overhead.
- Improves confidence that malformed env tokens do not reduce default security-header posture.

## Next

1. Add HSTS-enabled invalid-env fallback coverage (`M38-S70`) to lock default HSTS-disabled behavior under malformed `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED` values.
2. Continue security-header fallback slices until all top-level env toggles have explicit runtime contract tests.
