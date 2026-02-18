# M39: C Runtime Duplicate Header Merge Parity for `req.header` / `req.cookie`

## Why

LASM request parsing already merged duplicate request headers case-insensitively, but C runtime `req.header(...)` and `req.cookie(...)` still read only the first matching header line.

That created backend divergence for duplicate header/cookie request scenarios.

## What Changed

1. Added `sec4_rt_parse_header_value_merged(...)` in runtime C:
   - merges duplicate header values case-insensitively in arrival order,
   - uses `, ` separator for generic headers,
   - uses `; ` separator for `Cookie`,
   - preserves first-value behavior for `Host` / `Content-Length`.
2. Switched `sec4_rt_req_header(...)` and `sec4_rt_req_cookie(...)` to use merged header extraction.
3. Added clang-gated runtime harness coverage asserting duplicate-merge behavior for:
   - `req.header("X-Request-Id")`,
   - `req.cookie("mode")`,
   - `req.cookie("session")` across split `Cookie` lines.

## Validation

1. `cargo test -p sec4 --test json_output c_bin_runtime_req_header_and_cookie_merge_duplicate_headers_when_clang_available`
2. `cargo test -p sec4 --test json_output c_bin_runtime_req_query_decodes_percent_encoded_values_when_clang_available`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
