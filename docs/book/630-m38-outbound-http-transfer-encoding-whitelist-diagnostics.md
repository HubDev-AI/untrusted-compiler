# M38-S10 Outbound HTTP Transfer-Encoding Whitelist Diagnostics

## What it is

M38-S10 hardens outbound response parsing by enforcing a transfer-encoding whitelist (`chunked` only).

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

The runtime does not implement arbitrary transfer-encoding stacks. Accepting unsupported tokens can lead to ambiguous body parsing.

This slice makes parser behavior explicit:

- unsupported tokens fail deterministically,
- malformed mixed-token ordering remains a separate deterministic failure.

## How it works

1. During `Transfer-Encoding` token parsing:
   - only `chunked` is accepted,
   - non-`chunked` tokens return deterministic unsupported status.
2. Existing invalid ordering checks remain in place:
   - tokens after `chunked` still map to `NET.TRANSFER_ENCODING_INVALID`.
3. Outbound read-error mapping now includes:
   - `NET.TRANSFER_ENCODING_UNSUPPORTED`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_unsupported_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime rejects upstream responses that rely on unsupported transfer-encoding tokens.
- This is intentional for deterministic parser guarantees and reduced ambiguity in outbound dependency handling.

## Next

1. Continue parser hardening for obs-fold/malformed whitespace diagnostics.
2. Keep focused outbound runtime hardening tests green on each slice.
