# M38-S28 Outbound HTTP Redirect Policy Harness Expansion

## What it is

M38-S28 expands runtime harness coverage for field-specific redirect policy diagnostics, completing dedicated test coverage across all redirect policy keys.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Field-specific policy diagnostics were introduced, but dedicated coverage for revalidate/downgrade invalid-value paths was missing. This slice closes that coverage gap.

## How it works

1. Added dedicated harness for revalidate policy invalid path:
   - `NET.REDIRECT_POLICY_REVALIDATE_REDIRECTS_INVALID`
2. Added dedicated harness for allow-downgrade policy invalid path:
   - `NET.REDIRECT_POLICY_ALLOW_DOWNGRADE_INVALID`
3. Revalidated existing max-redirects policy-invalid path:
   - `NET.REDIRECT_POLICY_MAX_REDIRECTS_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_revalidate_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allow_downgrade_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_max_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Test harness surface grows with policy-key granularity.
- The additional test cost is small compared to improved deterministic coverage guarantees.

## Next

1. Harden policy invalid diagnostics with structured detail fields for easier machine parsing.
2. Keep redirect policy and hop diagnostics green while adding richer error envelopes.
