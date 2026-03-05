# M39: `req.httpVersion` Intrinsic and LASM Placeholder Parity

## Why

Method/path/cookie/query/header access was available, but HTTP version was still not first-class.

That limited request metadata parity for handlers and blocked deterministic response materialization based on request protocol version.

## What Changed

1. Added `req.httpVersion()` / `req_http_version()` intrinsic support in semantic typing and signature enforcement (zero-argument call).
2. Added C backend lowering rewrite to `sec4_rt_req_http_version()`.
3. Extended runtime C request state with `http_version` capture from the parsed request line and added:
   - `sec4_rt_req_http_version(void)` ABI surface.
4. Extended LASM placeholder extraction/materialization with:
   - intrinsic call extraction for `req.httpVersion()`,
   - literal token support for `{{req.httpVersion}}`.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols`
2. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
4. `cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available`
