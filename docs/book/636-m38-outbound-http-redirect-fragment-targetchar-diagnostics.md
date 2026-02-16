# M38-S16 Outbound HTTP Redirect Fragment/Target-Character Diagnostics

## What it is

M38-S16 hardens redirect target validation by splitting fragment-related failures from invalid target-character failures and mapping both to deterministic runtime diagnostics.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect target validation still collapsed fragment-bearing targets and character-invalid targets into broader redirect-invalid paths. This made diagnostics less precise and reduced policy/debug clarity.

This slice introduces dedicated outcomes and deterministic codes for both cases.

## How it works

1. Redirect resolver status expansion:
   - adds dedicated resolve statuses for fragment-invalid and target-character-invalid targets.
2. Relative target validation split:
   - `#` now maps to fragment-invalid status,
   - control characters/space/backslash now map to target-character-invalid status.
3. Runtime error mapping:
   - fragment-invalid resolves to `NET.REDIRECT_FRAGMENT_INVALID`,
   - target-character-invalid resolves to `NET.REDIRECT_TARGET_CHAR_INVALID`.
4. Existing host validation and relative-path normalization remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_fragment_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_target_char_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_host_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_relative_redirect_is_normalized_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now rejects redirect targets with stricter split diagnostics, which may surface failures earlier for previously tolerated malformed inputs.
- This is intentional to preserve deterministic security behavior and clearer remediation paths.

## Next

1. Continue redirect hardening with explicit malformed query-component target diagnostics.
2. Keep redirect parser matrix tests green while expanding deterministic error coverage.
