# M38-S31 Outbound HTTP Policy Diagnostics Cleanup

## What it is

M38-S31 performs a cleanup pass on redirect policy parsing internals by removing an unused generic strict-int parser helper while preserving behavior.

Files:

- `runtime/c/sec4_runtime.c`

## Why it exists

After max-redirects parsing was specialized for format/range diagnostics, the old generic strict-int parser helper became dead code. Keeping it would increase maintenance noise.

## How it works

1. Removed unused helper declaration and implementation:
   - `sec4_rt_parse_env_non_negative_i64_strict`
2. Retained active redirect policy parsing path with explicit deterministic logic.
3. Verified no behavior change in policy diagnostics.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_max_redirects_policy_range_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_revalidate_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- No functional trade-off; this is a maintenance/readability improvement.
- Smaller helper surface makes subsequent policy hardening changes clearer.

## Next

1. Run final naming-consistency sweep across redirect policy diagnostics.
2. Execute targeted redirect matrix pass before moving to the next alpha readiness block.
