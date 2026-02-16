# M38-S12 Outbound HTTP Header-Control/Section Diagnostics

## What it is

M38-S12 adds deterministic diagnostics for malformed response header sections and invalid control characters in header values.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Generic response-invalid errors made parser failures less actionable. This slice splits two concrete failure classes:

- malformed header-section framing,
- invalid control bytes in header values.

## How it works

1. Header-section framing:
   - missing/invalid header terminator patterns now map to `NET.HEADER_SECTION_INVALID`.
2. Header-value control chars:
   - values containing disallowed control bytes map to `NET.HEADER_VALUE_CONTROL_INVALID`.
3. Existing parser hardening paths (obs-fold, header whitespace, transfer-encoding strictness) remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_invalid_header_section_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_header_value_control_char_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_obs_fold_header_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_unsupported_transfer_encoding_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime rejects additional malformed upstream responses that may have previously collapsed into generic parse errors.
- This improves incident triage and deterministic behavior at the cost of stricter compatibility with non-compliant dependencies.

## Next

1. Continue parser hardening with content-type token strictness diagnostics.
2. Keep focused outbound hardening tests green per slice.
