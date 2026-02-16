# M38-S73 Security-Headers CSP Policy Invalid Env Fallback

## What it is

M38-S73 adds runtime e2e coverage proving invalid `SEC4_RT_SECURITY_HEADERS_CSP_POLICY` env values deterministically fall back to the default CSP policy.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Runtime CSP policy loading validates header values (rejecting unsafe values such as CR/LF). That fallback behavior needed explicit e2e contract coverage so malformed env input cannot inject invalid header payloads or change CSP policy unpredictably.

## How it works

1. Added clang-gated runtime e2e test:
   - `c_bin_http_runtime_applies_security_headers_csp_policy_invalid_env_falls_back_to_default_policy_when_enabled`
2. The harness starts a c-bin HTTP service with:
   - `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED=1`
   - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY=0`
   - invalid CSP policy env value containing newline.
3. Test asserts deterministic fallback behavior:
   - response includes default CSP policy header
   - response does not reflect invalid policy payload.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_policy_invalid_env_falls_back_to_default_policy_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one focused fallback e2e test and minor runtime-suite overhead.
- Improves confidence that malformed env input cannot degrade CSP header safety guarantees.

## Next

1. Add CORS allowed-origins invalid-env fallback coverage (`M38-S74`) for deterministic wildcard-origin baseline behavior under malformed env input.
2. Continue policy/env hardening slices with explicit runtime contract tests.
