# M38-S6 Outbound HTTP Parser Token/Status Strictness

## What it is

M38-S6 adds stricter outbound HTTP parser validation for status-line shape and transfer-encoding token ordering.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Ambiguous status-line parsing and permissive transfer-encoding token handling can hide malformed upstream responses and make runtime behavior less predictable.

This slice upgrades parser strictness with deterministic diagnostics for both failure classes.

## How it works

1. Status-line parser now requires:
   - protocol prefix `HTTP/1.0` or `HTTP/1.1`,
   - required protocol separator space,
   - strict 3-digit status code formatting.
2. Transfer-encoding parser now rejects token payload after `chunked`, enforcing `chunked` as terminal framing token.
3. Outbound read-error mapping adds deterministic codes:
   - `NET.STATUS_LINE_INVALID`
   - `NET.TRANSFER_ENCODING_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_status_line_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Stricter parser behavior may reject non-compliant upstream services that previously passed.
- This is intentional to keep runtime semantics deterministic and reduce parse ambiguity in dependency calls.

## Next

1. Continue M38 with response-version and header-line normalization diagnostics.
2. Keep focused outbound runtime harness coverage for parser hardening slices.
