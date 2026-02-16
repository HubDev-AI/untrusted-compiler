# M38-S11 Outbound HTTP Header-Whitespace/Obs-Fold Diagnostics

## What it is

M38-S11 strengthens outbound response-header parsing with dedicated diagnostics for obs-fold and header-name whitespace violations.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Obs-fold and malformed header-name whitespace create ambiguous parsing behavior and are disallowed in modern HTTP processing.

This slice splits these failure modes into explicit deterministic runtime codes.

## How it works

1. Response header lines that begin with SP/HTAB (obs-fold continuation) are rejected:
   - code: `NET.HEADER_OBS_FOLD_INVALID`
2. Header names with whitespace before `:` are rejected:
   - code: `NET.HEADER_WHITESPACE_INVALID`
3. Existing generic header-line invalid diagnostics still cover missing colon and invalid header-name tokens.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_obs_fold_header_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_header_whitespace_before_colon_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_unsupported_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now rejects additional non-compliant upstream responses.
- This is intentional for deterministic parser behavior and clearer incident diagnostics.

## Next

1. Continue parser hardening for header-value control-character diagnostics.
2. Keep focused runtime parser matrix green per slice.
