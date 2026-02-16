# M38-S9 Outbound HTTP Duplicate-Content-Length Diagnostics

## What it is

M38-S9 hardens outbound response parsing for duplicate `Content-Length` header handling.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Some upstream responses emit comma-delimited or repeated `Content-Length` values. These are safe only when values agree; conflicts are ambiguous.

This slice makes the behavior deterministic:

- equal duplicate values are normalized and accepted,
- malformed/conflicting values are rejected.

## How it works

1. `Content-Length` parsing now supports comma-delimited numeric tokens.
2. All parsed tokens must be valid integers and equal.
3. Any malformed token or conflicting value triggers deterministic `NET.CONTENT_LENGTH_INVALID`.
4. Existing transfer-encoding/content-length framing conflict checks remain active.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_duplicate_content_length_equal_is_accepted_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_duplicate_content_length_conflict_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_retry_after_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Parser strictness increases for malformed upstream metadata.
- This improves deterministic safety and observability while potentially rejecting previously tolerated non-compliant responses.

## Next

1. Continue outbound parser hardening around transfer-encoding token whitelist/mixed-token diagnostics.
2. Keep focused runtime harness coverage for each parser security slice.
