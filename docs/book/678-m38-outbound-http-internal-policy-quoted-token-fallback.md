# M38-S57 Outbound HTTP Internal-Policy Quoted-Token Fallback

## What it is

M38-S57 adds direct runtime harness coverage for quoted and escaped token spellings on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness proves quoted token spellings do not bypass internal-net deny-by-default behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S56 covered unset/blank token states. Quoted token spellings remained a separate normalization edge.

This slice ensures non-canonical quoted forms cannot accidentally enable internal net access:

- quoted token spellings must remain deny-by-default
- fallback outcome must stay `NET.INTERNAL_DENIED`

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_quoted_tokens_fallback_to_deny_when_clang_available`
2. Harness executes `sec4_rt_http_get_internal(1, valid_internal_url)` and asserts:
   - `NET.INTERNAL_DENIED`
   - `kind=authorization`
3. It validates quoted/escaped token classes:
   - double-quoted (`"\"1\""`, `"\"true\""`, `"\"allow\""`)
   - single-quoted (`"'yes'"`, `"'on'"`, `"'ALLOW'"`)
   - quoted with inner whitespace (`"\" TRUE \""`, `"' yes '"`)
4. It asserts no diagnostic drift into:
   - `NET.GET_INTERNAL_INVALID`
   - `NET.URL_INTERNAL_INVALID`
   - `NET.REQUEST_*`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_quoted_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_empty_or_whitespace_tokens_fallback_to_deny_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds another focused contract test without runtime code changes.
- Increases harness count slightly while making token-normalization boundaries explicit.

## Next

1. Add delimited-token fallback coverage (for example `"true,allow"`, `"yes|on"`, `"allow/1"`) to keep policy token parsing strict.
2. Continue locking internal-policy token behavior as small deterministic slices.
