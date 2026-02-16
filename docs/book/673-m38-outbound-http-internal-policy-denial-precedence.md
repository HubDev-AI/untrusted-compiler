# M38-S52 Outbound HTTP Internal-Policy Denial Precedence

## What it is

M38-S52 adds direct runtime harness coverage for internal-net policy denial precedence on valid internal URL handles.

The harness validates `sec4_rt_http_get_internal` deterministic denial behavior when policy is disabled.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S51 locked invalid-handle wrapper diagnostics. The next boundary is valid internal URL handles under policy deny.

This slice ensures that policy denial remains the first-class deterministic outcome:

- code: `NET.INTERNAL_DENIED`
- kind: `authorization`

without drifting into URL/parser/handle failure classes for the same inputs.

## How it works

1. Added a new clang-gated harness test:
   - `c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available`
2. The harness builds valid internal URL handles and calls `sec4_rt_http_get_internal(1, url)` for:
   - `http://127.0.0.1/internal-a`
   - `http://localhost/internal-b`
3. The harness asserts deterministic policy-deny envelopes and precedence exclusions:
   - includes `NET.INTERNAL_DENIED`, `kind=authorization`
   - excludes `NET.URL_INTERNAL_INVALID`
   - excludes `NET.REQUEST_*`
   - excludes `NET.GET_INTERNAL_INVALID`
4. The test runs in both deny states:
   - default deny (env removed)
   - explicit deny (`SEC4_RT_ALLOW_INTERNAL_NET=0`)

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused wrapper-policy contract harness without changing runtime C logic.
- Increases targeted harness count, but keeps failure classes separated and regression diagnosis clearer.

## Next

1. Cover truthy-token policy-allow behavior for `SEC4_RT_ALLOW_INTERNAL_NET` and ensure denial is bypassed deterministically.
2. Keep wrapper-level policy, invalid-handle, invalid-URL, and parser diagnostics as separate contract tests.
