# M38-S30 Outbound HTTP Redirect Max-Redirects Format/Range Diagnostics

## What it is

M38-S30 refines redirect policy diagnostics by splitting `SEC4_RT_NET_PUBLIC_MAX_REDIRECTS` failures into separate deterministic format-invalid and range-invalid codes.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`MAX_REDIRECTS` had a single invalid code, which conflated malformed numeric format and out-of-range values. Splitting these paths improves operator clarity and deterministic triage.

## How it works

1. `SEC4_RT_NET_PUBLIC_MAX_REDIRECTS` is parsed directly with explicit checks:
   - non-numeric format -> `NET.REDIRECT_POLICY_MAX_REDIRECTS_FORMAT_INVALID`
   - numeric but out of allowed range -> `NET.REDIRECT_POLICY_MAX_REDIRECTS_RANGE_INVALID`
2. Both paths retain structured details:
   - `details:[{"key":"policyKey","value":"SEC4_RT_NET_PUBLIC_MAX_REDIRECTS"}]`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_max_redirects_policy_range_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_max_redirects_policy_format_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_revalidate_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Redirect policy parser logic is slightly more explicit and verbose.
- The added branching is worth it for deterministic, field-precise diagnostics.

## Next

1. Clean up redirect policy helper surface and remove now-redundant generic strict-int parser paths.
2. Keep redirect policy diagnostic matrix green while tightening parser internals.
