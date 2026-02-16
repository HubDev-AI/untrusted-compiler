# M38-S65 Security-Headers Referrer-Policy Invalid Env Fallback

## What it is

M38-S65 implements and validates deterministic fallback behavior for invalid `SEC4_RT_SECURITY_HEADERS_REFERRER_POLICY` env values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Runtime previously copied `SEC4_RT_SECURITY_HEADERS_REFERRER_POLICY` directly from env without validating whether the value was an allowed policy token.

That left room for malformed header values to be emitted as-is. This slice closes that gap by clamping invalid values to a safe default.

## How it works

1. Added runtime allowlist validator:
   - `sec4_rt_is_referrer_policy_valid(...)`
2. In `sec4_rt_load_security_headers_policy_from_env(...)`, after reading env value:
   - invalid values now clamp to `strict-origin-when-cross-origin`
3. Added clang-gated HTTP runtime e2e coverage:
   - `c_bin_http_runtime_applies_security_headers_referrer_policy_invalid_env_falls_back_to_default_when_enabled`
4. Harness runs with invalid env token `INVALID-POLICY` and asserts:
   - response contains `Referrer-Policy: strict-origin-when-cross-origin`
   - response does not reflect `INVALID-POLICY`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_referrer_policy_invalid_env_falls_back_to_default_when_enabled`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds a small runtime validation helper and one focused e2e contract test.
- Keeps behavior deterministic with an explicit allowlist, but any future policy-token additions must also update the runtime allowlist.

## Next

1. Add x-content-type-options invalid-env fallback coverage (`M38-S66`) to lock boolean fallback semantics for `nosniff`.
2. Continue security-header fallback hardening until all env-driven header branches have explicit e2e regression contracts.
