# M38-S7 Outbound HTTP Response-Version/Header-Line Diagnostics

## What it is

M38-S7 adds deterministic diagnostics for unsupported response HTTP versions and malformed response header lines on outbound runtime parsing.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Even after status-line and transfer-encoding strictness, parser behavior still accepted some malformed response metadata that should fail predictably.

This slice closes that gap with explicit runtime codes for response-version and header-line shape failures.

## How it works

1. Runtime now distinguishes unsupported response versions from generic status-line errors:
   - non-`HTTP/1.0` / `HTTP/1.1` response prefixes map to `NET.RESPONSE_VERSION_UNSUPPORTED`.
2. Runtime validates each response header line before field-specific parsing:
   - rejects folded/leading-whitespace header lines,
   - requires `name:value` shape,
   - enforces token-safe header-name characters.
3. Invalid header-line shape maps to deterministic `NET.HEADER_LINE_INVALID`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_unsupported_version_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_header_line_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_status_line_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now rejects a wider set of malformed upstream responses that might previously pass.
- This improves deterministic security/observability semantics at the cost of stricter compatibility with non-compliant dependencies.

## Next

1. Continue outbound parser hardening with duplicate-header normalization and retryability diagnostics.
2. Keep focused clang-gated harness tests for parser-security slices.
