# M38-S8 Outbound HTTP Duplicate-Header/Retryability Diagnostics

## What it is

M38-S8 hardens outbound response-header parsing for duplicate `Location` and `Retry-After` behavior.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Duplicate headers with conflicting values can make redirect/retry semantics ambiguous and nondeterministic.

This slice makes both paths deterministic with explicit failure codes.

## How it works

1. `Location` normalization:
   - duplicate `Location` values are allowed only when values match exactly,
   - conflicting duplicate values are rejected as `NET.REDIRECT_LOCATION_CONFLICT`.
2. `Retry-After` normalization:
   - value must be numeric seconds,
   - malformed values or conflicting duplicates are rejected as `NET.RETRY_AFTER_INVALID`.
3. Existing redirect/chunked/status-line/header-line behavior remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_conflicting_location_headers_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_retry_after_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_unsupported_version_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_header_line_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime is stricter with non-compliant upstream responses carrying conflicting duplicate metadata.
- This is intentional for deterministic parser behavior and clearer operational diagnostics.

## Next

1. Continue parser hardening around duplicate `Content-Length` normalization and malformed-header diagnostics.
2. Keep the outbound runtime hardening matrix green on each slice.
