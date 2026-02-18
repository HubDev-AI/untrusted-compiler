# 890 M38 Slice: Runtime `req.pathParam` Percent-Decoding

This slice extends runtime request decoding parity by adding percent-decoding for route path parameters.

## What changed

Path parameter extraction now decodes:

- `%XX` hex escapes
- `+` into space

Implementation is in `runtime/c/sec4_runtime.c` inside `sec4_rt_extract_path_param(...)`.

## Behavior details

- Valid escaped path-segment values are decoded before being tracked.
- Invalid escape sequences preserve deterministic fallback behavior by returning the raw segment value.
- Missing path-parameter names preserve existing fallback semantics in `sec4_rt_req_path_param(...)`.

## Validation

Added targeted runtime harness coverage in:

- `compiler/sec4-cli/tests/json_output.rs`
  - `c_bin_runtime_req_path_param_decodes_percent_encoded_values_when_clang_available`

Validation run:

- `cargo test -p sec4 --test json_output c_bin_runtime_req_path_param_decodes_percent_encoded_values_when_clang_available`

Passed.
