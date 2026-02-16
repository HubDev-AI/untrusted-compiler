# M38-S14 Outbound HTTP Redirect-Target Normalization Diagnostics

## What it is

M38-S14 hardens outbound redirect handling by normalizing relative redirect targets and splitting invalid-target diagnostics.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect handling previously used a coarse success/fail path. Relative targets could be ambiguous without explicit normalization and dedicated failure reasons.

This slice makes redirect target processing deterministic and explicit.

## How it works

1. Redirect resolver now returns structured outcomes:
   - `OK`
   - `INVALID`
   - `TARGET_INVALID`
2. Relative targets are normalized with dot-segment handling (`.`/`..`) before follow-up request construction.
3. Targets that escape root or contain invalid payload map to deterministic:
   - `NET.REDIRECT_TARGET_INVALID`
4. Existing `NET.REDIRECT_INVALID` behavior remains for non-target-specific resolve failures.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_relative_redirect_is_normalized_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_relative_redirect_invalid_target_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_conflicting_location_headers_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now rejects additional malformed relative redirect targets.
- This is intentional for deterministic URL normalization and reduced ambiguity in outbound redirect execution.

## Next

1. Continue redirect hardening with authority/host-token diagnostics.
2. Keep the focused outbound parser matrix green on every slice.
