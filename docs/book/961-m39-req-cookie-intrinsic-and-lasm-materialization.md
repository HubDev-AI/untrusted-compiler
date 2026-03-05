# M39: `req.cookie` Intrinsic + LASM Materialization Support

## Why

The request intrinsic surface supported `req.query`, `req.pathParam`, and `req.header`, but had no first-class cookie accessor.

That blocked direct cookie-based handler logic and left LASM dynamic materialization without parity for cookie-derived placeholders.

## What Changed

1. Added `req.cookie` / `req_cookie` to semantic intrinsic typing (`Untrusted<String>`, `net` effect) and request-signature enforcement.
2. Added C backend lowering rewrites for `req.cookie(...) -> sec4_rt_req_cookie(...)`.
3. Added runtime ABI + implementation:
   - `runtime/c/sec4_runtime.h`: `sec4_rt_req_cookie(const char *name)`,
   - `runtime/c/sec4_runtime.c`: reads `Cookie` header and parses cookie value deterministically using existing cookie parser.
4. Extended LASM dynamic placeholder extraction/materialization to support `req.cookie("...")` and `{{req.cookie:...}}` in response body/header/cookie flows.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols`
2. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
4. `cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available`
