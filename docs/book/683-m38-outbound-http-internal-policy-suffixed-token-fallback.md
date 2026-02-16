# M38-S62 Outbound HTTP Internal-Policy Suffixed-Token Fallback

## What it is

M38-S62 adds direct runtime harness coverage for suffixed token spellings on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves suffix-decorated token values do not bypass internal-net deny-by-default policy.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S59 covered prefixed spellings, and M38-S58 covered delimited forms. Suffixed token forms were still an acceptance-boundary gap.

This slice ensures suffix-decorated token values stay invalid and deterministic:

- suffixed token spellings must not enable internal net access
- fallback outcome must remain `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_suffixed_tokens_fallback_to_deny_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates suffixed token classes:
   - hyphen/underscore suffixes (`"true-value"`, `"allow_mode"`, `"yes-end"`)
   - alphanumeric/punctuation suffixes (`"on1"`, `"allow+"`, `"1ok"`, `"true."`)
4. It asserts no diagnostic drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_suffixed_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_prefixed_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused contract harness without changing runtime implementation.
- Increases token-boundary test coverage while keeping the policy parser strict and predictable.

## Next

1. Add HSTS invalid-env fallback coverage (`M38-S63`) for deterministic safe defaults on malformed max-age values.
2. Continue prioritizing runtime behavior slices that improve no-stub alpha evidence quality.
