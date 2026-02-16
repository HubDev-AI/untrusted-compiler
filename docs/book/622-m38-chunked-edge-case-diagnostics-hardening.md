# M38-S2 Chunked Edge-Case Diagnostics Hardening

## What it is

M38-S2 adds explicit runtime diagnostics for malformed chunked outbound responses.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

After M38-S1 chunked decoding, malformed chunk frames were still reported through a generic outbound-response error path.

This slice makes chunk framing failures explicit and deterministic for debugging and policy/audit clarity.

## How it works

1. Chunked parser failures now return a dedicated internal read status (`-7`) for invalid chunk framing.
2. Outbound read-error mapping handles `-7` as:
   - `code: NET.CHUNK_INVALID`
   - deterministic message: `outbound http chunked response framing is invalid`
3. Existing limit and timeout mappings remain unchanged:
   - `NET.RESPONSE_TOO_LARGE`
   - `NET.READ_TIMEOUT`
   - `NET.RESPONSE_TRACK_FAILED`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_malformed_chunked_returns_chunk_invalid_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`

## Trade-offs

- This keeps error taxonomy small by adding one chunk-specific code instead of many parser sub-codes.
- It improves observability without changing supported protocol scope.

## Next

1. Expand chunked edge coverage for trailers/extensions-heavy responses (M38-S3).
2. Keep redirect and TLS targeted regression tests in lockstep with outbound parser changes.
