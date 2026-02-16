# M38-S60 Security-Headers CSP Runtime Materialization

## What it is

M38-S60 implements real CSP header materialization in the C runtime security-headers path.

Runtime now carries CSP policy fields in security-header policy/router state and emits either enforce-mode or report-only CSP headers on HTTP responses.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`sec.defaultHeaders()` and `sec.withSecurityHeaders(...)` already applied `nosniff`, `x-frame-options`, and `referrer-policy`, but CSP materialization was missing from runtime response headers.

This slice closes that gap and makes CSP behavior real for v0.1 runtime flows.

## How it works

1. Extended runtime policy/router state with CSP fields:
   - `security_csp_enabled`
   - `security_csp_report_only`
   - `security_csp_policy`
2. Added CSP env policy loading in `sec4_rt_load_security_headers_policy_from_env`:
   - `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED`
   - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY`
   - `SEC4_RT_SECURITY_HEADERS_CSP_POLICY`
3. Added safe fallback for invalid CSP header values to default policy:
   - `default-src 'self'; frame-ancestors 'none'; base-uri 'self'`
4. Updated `sec4_rt_security_headers_block` to emit:
   - `Content-Security-Policy: ...` (enforce mode), or
   - `Content-Security-Policy-Report-Only: ...` (report-only mode)
5. Updated e2e runtime tests:
   - existing success-path security headers test now asserts default CSP header
   - new report-only test asserts report-only header selection and no enforce-mode header for the same policy string

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_report_only_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds state fields and header rendering logic in runtime security middleware path, increasing code surface slightly.
- Keeps CSP policy materialization deterministic and environment-driven without introducing runtime policy parsing complexity beyond existing env model.

## Next

1. Add suffixed-token fallback coverage for `SEC4_RT_ALLOW_INTERNAL_NET` (`M38-S61`) to continue locking internal-policy token parsing boundaries.
2. Expand CSP test coverage to error-path responses under report-only mode for full middleware parity.
