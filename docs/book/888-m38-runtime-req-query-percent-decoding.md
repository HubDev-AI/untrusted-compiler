# 888 M38 Slice: Runtime `req.query` Percent-Decoding

This slice improves real runtime behavior for query-string handling.

## What changed

`sec4_rt_req_query(...)` now decodes standard URL query encodings before tracking the returned value:

- `%XX` hex escapes are decoded (for example `%2F` -> `/`).
- `+` is decoded to space.
- query key matching accepts percent-encoded keys (for example `na%6De` matches `name`).

Implementation lives in `runtime/c/sec4_runtime.c` and is wired through `sec4_rt_extract_query_value(...)`.

## Compatibility behavior

- Valid encoded values are decoded.
- Invalid percent-escape sequences keep previous practical behavior by falling back to raw segment matching/value extraction (for example `bad%zz` stays `bad%zz`).
- Missing query keys still preserve existing fallback behavior in `sec4_rt_req_query` (returns tracked key name).

## Validation

Added targeted runtime harness coverage in:

- `compiler/sec4-cli/tests/json_output.rs`
  - `c_bin_runtime_req_query_decodes_percent_encoded_values_when_clang_available`

Validation run:

- `cargo test -p sec4 --test json_output c_bin_runtime_req_query_decodes_percent_encoded_values_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_path_and_header_guards_when_clang_available`

Both passed.
