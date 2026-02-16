# M38-S58 Outbound HTTP Internal-Policy Delimited-Token Fallback

## What it is

M38-S58 adds direct runtime harness coverage for delimited token spellings on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves delimiter-combined token values do not bypass internal-net deny-by-default policy.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S57 covered quoted spellings. Delimiter-combined token forms remained an input-normalization edge.

This slice ensures combined token strings are treated as invalid and stay deny-by-default:

- delimited token spellings must not enable internal net access
- fallback outcome must remain `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_delimited_tokens_fallback_to_deny_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates delimiter token classes:
   - comma (`"true,allow"`, `"allow,true"`)
   - pipe/slash (`"yes|on"`, `"allow/1"`)
   - semicolon/colon (`"true;allow"`, `"on:yes"`)
   - delimiter + spacing (`" yes|allow "`)
4. It asserts no diagnostic drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_delimited_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_quoted_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one more contract-harness test without changing runtime implementation.
- Slightly increases test count while tightening token-normalization guarantees.

## Next

1. Add prefixed-token fallback coverage (for example `"allow=true"`, `"mode:allow"`, `"token=1"`) to keep parser acceptance strict.
2. Keep internal-policy token behavior locked in small deterministic slices for faster regression triage.
