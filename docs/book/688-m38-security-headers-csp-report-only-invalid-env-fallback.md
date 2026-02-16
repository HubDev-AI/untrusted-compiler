# M38-S67 Security-Headers CSP Report-Only Invalid Env Fallback

## What it is

M38-S67 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY` env values fall back to enforce-mode CSP header behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

CSP report-only mode is controlled by strict env boolean parsing. We already covered explicit report-only enable behavior, but malformed-token fallback behavior was not explicitly locked in a dedicated runtime contract test.

This slice ensures invalid boolean env values cannot accidentally switch runtime to report-only mode.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_csp_report_only_invalid_env_falls_back_to_enforce_when_enabled`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED=1`
   - `SEC4_RT_SECURITY_HEADERS_CSP_POLICY=default-src 'self'; object-src 'none'`
   - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY=MAYBE` (invalid)
3. Test asserts deterministic fallback behavior:
   - response includes `Content-Security-Policy: ...`
   - response does not include `Content-Security-Policy-Report-Only: ...`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_report_only_invalid_env_falls_back_to_enforce_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_report_only_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one targeted e2e fallback test and small runtime-suite overhead.
- Improves determinism guarantees for CSP mode behavior under malformed env inputs.

## Next

1. Add CSP enabled invalid-env fallback coverage (`M38-S68`) to lock default-on semantics for malformed `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED` values.
2. Continue security-header hardening until each env-driven branch has explicit runtime contract coverage.
