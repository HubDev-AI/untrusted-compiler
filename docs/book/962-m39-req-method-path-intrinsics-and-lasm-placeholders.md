# M39: `req.method` + `req.path` Intrinsics and LASM Placeholders

## Why

Request metadata access lacked dedicated intrinsics for HTTP method and normalized path.

That forced handler logic to infer request shape indirectly and left LASM response materialization without first-class method/path placeholder parity.

## What Changed

1. Added semantic support for:
   - `req.method()` / `req_method()`,
   - `req.path()` / `req_path()`.
2. Enforced zero-argument signatures for both intrinsics (`E4001` on unexpected arguments).
3. Added C backend rewrites:
   - `req.method(...) -> sec4_rt_req_method(...)`,
   - `req.path(...) -> sec4_rt_req_path(...)`.
4. Added runtime ABI + implementation:
   - `sec4_rt_req_method(void)`,
   - `sec4_rt_req_path(void)` (prefers normalized route path when available).
5. Extended LASM placeholder extraction/materialization with:
   - `req.method()` and `req.path()` call extraction,
   - literal placeholders `{{req.method}}` and `{{req.path}}` for body/header/cookie materialization paths.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols`
2. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
4. `cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available`
