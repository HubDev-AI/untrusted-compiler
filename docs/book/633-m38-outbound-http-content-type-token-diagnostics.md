# M38-S13 Outbound HTTP Content-Type Token Diagnostics

## What it is

M38-S13 adds deterministic validation for outbound response `Content-Type` media-type tokens.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Malformed `Content-Type` values can leak parser ambiguity and make downstream behavior inconsistent.

This slice introduces a strict type/subtype token check and explicit deterministic failure code.

## How it works

1. During header parsing, runtime inspects `Content-Type` values.
2. The media type token must match `type/subtype` structure with allowed token characters.
3. Invalid/malformed values map to deterministic `NET.CONTENT_TYPE_INVALID`.
4. Existing outbound parser hardening remains unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_content_type_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_header_section_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_obs_fold_header_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now rejects malformed upstream `Content-Type` headers that may have been tolerated previously.
- This is intentional for deterministic parser behavior and clearer diagnostics.

## Next

1. Continue parser hardening around redirect-location normalization diagnostics.
2. Keep focused outbound parser matrix coverage per slice.
