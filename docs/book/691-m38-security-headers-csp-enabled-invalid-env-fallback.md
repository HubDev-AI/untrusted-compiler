# M38-S68 Security-Headers CSP Enabled Invalid Env Fallback

## What it is

M38-S68 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED` env values deterministically keep CSP enabled.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CSP enable/disable is controlled by strict env-boolean parsing. Existing runtime behavior already defaults to enabled CSP when parsing fails, but this fallback was not covered by a dedicated e2e contract test.

This slice locks that behavior to prevent regressions where malformed env values could silently disable CSP.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_csp_enabled_invalid_env_falls_back_to_enabled_when_security_headers_enabled`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED=MAYBE` (invalid)
   - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY=0`
   - explicit CSP policy value.
3. Test asserts deterministic fallback behavior:
   - response includes enforce-mode `Content-Security-Policy` header
   - response does not include report-only CSP header.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_enabled_invalid_env_falls_back_to_enabled_when_security_headers_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused e2e fallback test and minor runtime-suite overhead.
- Improves confidence that invalid env tokens cannot reduce CSP posture.

## Next

1. Add security-headers middleware-enabled invalid-env fallback coverage (`M38-S69`) so invalid `SEC4_RT_SECURITY_HEADERS_ENABLED` tokens deterministically preserve baseline header posture.
2. Continue converting env-driven security-header assumptions into explicit runtime contract tests.
