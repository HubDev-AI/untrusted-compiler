# M38-S54 Outbound HTTP Internal-Policy Invalid-Token Fallback

## What it is

M38-S54 adds direct runtime harness coverage for unknown-token fallback behavior on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves invalid policy tokens preserve deny-by-default semantics for internal outbound requests.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S53 covered accepted truthy allow tokens. Remaining ambiguity was invalid token behavior.

This slice makes invalid-token fallback explicit and deterministic:

- invalid tokens must not silently allow internal net access
- fallback outcome must remain `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_invalid_tokens_fallback_to_deny_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` for invalid token values and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates multiple invalid token classes:
   - semantic noise (`"maybe"`, `"enabled"`, `"true-ish"`)
   - numeric noise (`"2"`, `"-1"`)
   - near-miss/spacing noise (`"t"`, `"y"`, `" allow "`, `"YES!"`)
4. It asserts no fallback drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_invalid_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused policy-token harness without runtime code changes.
- Increases runtime contract test count but keeps policy-token behavior isolated and regressions easy to localize.

## Next

1. Add explicit deny-token matrix coverage (`0|false|no|off|deny`) with case-variance checks.
2. Keep allow-token, invalid-token fallback, and deny-precedence behavior as separate contract slices.
