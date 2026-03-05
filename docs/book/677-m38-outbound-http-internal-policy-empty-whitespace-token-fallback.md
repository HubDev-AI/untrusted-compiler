# M38-S56 Outbound HTTP Internal-Policy Empty/Whitespace-Token Fallback

## What it is

M38-S56 adds direct runtime harness coverage for unset, empty, and whitespace-only token behavior on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves blank token states preserve deny-by-default internal-net policy behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S55 locked explicit deny-token behavior, but blank token states still needed an explicit regression contract.

This slice ensures blank input forms cannot accidentally bypass internal-net policy:

- unset/blank token values must stay deny-by-default
- fallback outcome must remain `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_empty_or_whitespace_tokens_fallback_to_deny_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates token states:
   - unset (`env_remove`)
   - empty string (`""`)
   - whitespace-only strings (`" "`, `"   "`, `"\t"`, `"\n"`, `" \t "`)
4. It asserts no diagnostic drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_empty_or_whitespace_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_explicit_deny_tokens_enforce_denial_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one more focused contract test without changing runtime implementation.
- Slightly increases harness runtime while making policy-token fallback behavior more explicit and easier to maintain.

## Next

1. Add quoted-token fallback coverage (for example `"\"true\""`, `"'yes'"`) so only canonical allow tokens can bypass denial.
2. Keep internal-policy token behavior split into small deterministic slices to isolate regressions quickly.
