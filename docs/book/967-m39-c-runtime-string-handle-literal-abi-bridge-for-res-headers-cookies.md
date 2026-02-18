# M39: C Runtime String ABI Bridge for `res.text`, `headers.*`, and `cookie.build`

## Why

C backend emission mixes tracked string handles and C string literals at intrinsic call sites.

`res.text`, `headers.name/value`, and `cookie.build` previously required raw `const char *` inputs, which caused runtime termination when dynamic tracked values were passed.

## What Changed

1. Updated runtime ABI signatures to handle-based inputs:
   - `sec4_rt_res_text(int64_t status, int64_t body)`
   - `sec4_rt_cookie_build(int64_t name, int64_t value)`
   - `sec4_rt_headers_name(int64_t input)`
   - `sec4_rt_headers_value(int64_t input)`
2. Added shared runtime resolver `sec4_rt_resolve_tracked_or_literal_string(...)`:
   - prefers tracked-handle lookup,
   - falls back to literal pointer compatibility for legacy call shapes,
   - rejects obvious invalid low-address integer inputs.
3. Wired updated resolver into response/header/cookie helpers so dynamic request-derived values are handled safely in C runtime paths.
4. Added command integration coverage proving dynamic `req.header(...)` values can flow through `res.text(...)` under C backend oneshot execution.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
2. `cargo test -p sec4 --test commands run_command_oneshot_supports_dynamic_req_header_in_res_text`
3. `cargo test -p sec4 --test json_output c_bin_runtime_req_header_and_cookie_merge_duplicate_headers_when_clang_available`
