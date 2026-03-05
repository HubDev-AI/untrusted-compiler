# M38-S49 Outbound HTTP Parser Diagnostics Sink-Bridge Contract Coverage

## What it is

M38-S49 adds direct sink-bridge parity coverage for malformed outbound URL targets at wrapper boundaries.

The runtime harness now asserts that both:

- `sec4_rt_http_get` (public sink), and
- `sec4_rt_http_get_internal` (internal sink)

emit the same parser-class diagnostic contract when malformed target tokens reach outbound parser validation without redirect indirection.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S47 covered parser parity through redirect paths, but direct wrapper paths were still implicit.

This slice locks direct malformed-input behavior so parser diagnostics stay consistent across sink wrappers even when the failure happens before any network transport.

## How it works

1. Added a new clang-gated C runtime harness test:
   - `c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available`
2. The harness executes both wrappers on the same malformed URLs and asserts deterministic parser diagnostics:
   - `NET.REQUEST_TARGET_INVALID`
   - `{"key":"phase","value":"parse"}`
   - `{"key":"component","value":"target"}`
3. Cases include malformed targets that pass URL gate shape checks but fail outbound parser target validation:
   - fragment target (`#`)
   - CRLF contamination in target
   - query plus fragment target

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_rejects_invalid_url_or_untracked_handles_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- The parity lock focuses on parser-target malformed cases only; wrapper pre-parser invalid-url classes are still a separate contract surface.
- Adds one more clang-gated runtime harness, which slightly increases targeted runtime test time.

## Next

1. Cover wrapper pre-parser invalid-url envelope parity (`NET.URL_PUBLIC_INVALID` vs `NET.URL_INTERNAL_INVALID`) under equivalent malformed handles.
2. Keep parser-class and wrapper-class failures explicitly separated in runtime harnesses so regressions are easier to localize.
