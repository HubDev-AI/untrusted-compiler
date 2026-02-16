# M38-S3 Chunked Trailer/Extension Strictness

## What it is

M38-S3 tightens chunked-response parsing for outbound runtime HTTP:

- accepts chunk extensions and trailer headers on valid responses,
- requires explicit trailer termination after zero-size chunk,
- keeps deterministic `NET.CHUNK_INVALID` for invalid framing.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S2 introduced explicit malformed-chunk diagnostics, but parser behavior around trailer termination was still permissive.

This slice makes trailer handling strict while preserving compatibility with extension/trailer-bearing valid responses.

## How it works

1. In zero-size chunk handling, runtime now tracks trailer completion explicitly.
2. Success path requires either:
   - immediate `\r\n` terminator after `0\r\n`, or
   - trailer lines terminated by a final blank line.
3. If termination is missing, parser returns chunk-invalid read status (`-7`) which maps to `NET.CHUNK_INVALID`.
4. Extension and trailer-bearing valid responses decode to plain body bytes unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_trailers_roundtrip_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_missing_trailer_terminator_returns_chunk_invalid_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_malformed_chunked_returns_chunk_invalid_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`

## Trade-offs

- Parser strictness can reject non-compliant servers that previously happened to pass.
- This is intentional for deterministic behavior and safer runtime diagnostics.

## Next

1. Continue M38 with response-header/token edge-case hardening and diagnostics.
2. Keep focused runtime harness matrix for outbound parser changes.
