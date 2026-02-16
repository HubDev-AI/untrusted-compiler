# M38-S59 Outbound HTTP Internal-Policy Prefixed-Token Fallback

## What it is

M38-S59 adds direct runtime harness coverage for prefixed token spellings on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves key/value and namespace-like token forms do not bypass internal-net deny-by-default policy.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S58 covered delimiter-combined tokens. Prefixed token forms remained an additional normalization edge.

This slice ensures prefixed spellings stay invalid and deterministic:

- prefixed token spellings must not enable internal net access
- fallback outcome must remain `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_prefixed_tokens_fallback_to_deny_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates prefixed token classes:
   - key/value (`"allow=true"`, `"token=1"`, `"internal-net=on"`)
   - namespace/prefix (`"mode:allow"`, `"value:true"`, `"policy.allow=yes"`, `"allow:true"`)
4. It asserts no diagnostic drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_prefixed_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_delimited_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused contract test without changing runtime implementation.
- Increases test count slightly while tightening allowed-token parsing boundaries.

## Next

1. Add suffixed-token fallback coverage (for example `"true-value"`, `"allow_mode"`, `"yes-end"`) to keep token acceptance strict.
2. Continue locking policy-token behavior in narrow deterministic slices.
