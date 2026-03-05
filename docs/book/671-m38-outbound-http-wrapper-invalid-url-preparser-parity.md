# M38-S50 Outbound HTTP Wrapper Invalid-URL Pre-Parser Parity

## What it is

M38-S50 adds direct runtime harness coverage for wrapper-level invalid-url diagnostics before outbound parser execution.

The new harness validates deterministic envelope parity for:

- `sec4_rt_http_get` (public wrapper), and
- `sec4_rt_http_get_internal` (internal wrapper)

when malformed URL handles fail wrapper URL-gate validation.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S49 locked parser-class sink-bridge parity for malformed targets that reach outbound parser checks.

This slice closes the adjacent gap: malformed URL handles rejected at wrapper boundaries must keep deterministic wrapper-class diagnostics and must not leak into parser-class (`NET.REQUEST_*`) failures.

## How it works

1. Added a clang-gated harness test:
   - `c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available`
2. The harness calls both wrappers for equivalent malformed URL classes and asserts wrapper-class diagnostics:
   - public: `NET.URL_PUBLIC_INVALID`
   - internal: `NET.URL_INTERNAL_INVALID`
3. The harness also asserts:
   - `kind=validation` is present,
   - parser-class tokens (`NET.REQUEST_*`) are absent for wrapper-level failures,
   - untracked-handle failures map to the same wrapper-class diagnostics.

Malformed URL classes covered:

- invalid scheme (`ftp://...`)
- missing scheme (`example.com/...`)
- missing host (`http:///...`)
- userinfo authority token (`http://user@example.com/...`)

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- This slice adds one focused wrapper contract harness without changing runtime C behavior.
- Parser-class and wrapper-class contracts are now covered by separate tests, which improves failure localization but increases targeted harness count.

## Next

1. Add direct parity coverage for missing/zero capability handles:
   - `NET.GET_INVALID`
   - `NET.GET_INTERNAL_INVALID`
2. Keep wrapper-class and parser-class failures separated as independent regression contracts.
