# M38-S27 Outbound HTTP Redirect Policy Field-Specific Diagnostics

## What it is

M38-S27 replaces shared redirect policy-invalid diagnostics with field-specific deterministic error codes for each redirect policy input.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

A single generic `NET.REDIRECT_POLICY_INVALID` code made it harder to identify which specific policy field was malformed.

Field-specific codes provide better operator/actionability and deterministic triage.

## How it works

1. Redirect policy parsing now emits per-field deterministic codes:
   - `NET.REDIRECT_POLICY_ALLOW_REDIRECTS_INVALID`
   - `NET.REDIRECT_POLICY_MAX_REDIRECTS_INVALID`
   - `NET.REDIRECT_POLICY_REVALIDATE_REDIRECTS_INVALID`
   - `NET.REDIRECT_POLICY_ALLOW_DOWNGRADE_INVALID`
2. Existing redirect flow behavior remains unchanged after policy parse succeeds.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_max_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_cap_limit_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- More explicit diagnostics mean a slightly larger runtime error-code surface.
- The extra specificity is worth it for policy debugging and automated alert routing.

## Next

1. Add dedicated harnesses for remaining field-specific policy codes (`REVALIDATE_REDIRECTS`, `ALLOW_DOWNGRADE`).
2. Keep redirect hardening matrix tests green as policy diagnostics reach full field coverage.
