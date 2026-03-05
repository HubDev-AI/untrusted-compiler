# M38-S55 Outbound HTTP Internal-Policy Explicit Deny-Token Matrix

## What it is

M38-S55 adds direct runtime harness coverage for explicit deny tokens on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves explicit falsy/deny tokens always enforce internal-net policy denial across case variants.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S54 covered invalid-token fallback, but explicit deny-token behavior needed its own deterministic contract.

This slice makes deny-token handling explicit and regression-resistant:

- deny tokens must always keep internal net disabled
- deny-token results must stay on `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_explicit_deny_tokens_enforce_denial_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` for deny tokens and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates explicit deny tokens across case variants:
   - canonical: `"0"`, `"false"`, `"no"`, `"off"`, `"deny"`
   - case variants: `"FALSE"`, `"No"`, `"OFF"`, `"DeNy"`
4. It asserts no diagnostic drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_explicit_deny_tokens_enforce_denial_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_invalid_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused contract test slice without changing runtime implementation.
- Increases harness coverage and test time slightly, but makes internal-policy token behavior easier to debug and lock.

## Next

1. Add empty/whitespace-token fallback coverage (unset/blank values should preserve deny-by-default).
2. Keep allow-token, invalid-token, explicit-deny-token, and precedence behaviors isolated as separate contract slices.
