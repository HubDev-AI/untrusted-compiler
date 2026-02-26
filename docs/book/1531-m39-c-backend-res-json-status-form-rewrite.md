# 1531 M39 Slice: C Backend `res.json(status, schema, value)` Lowering Fix

This slice fixes a C emit/runtime ABI mismatch on strict-mode status-form JSON responses.

## What changed

1. Updated C backend lowering in `compiler/sec4-core/src/c_backend.rs`:
   - detect `res.json(...)` calls lowered to intrinsic marker form,
   - when argument count is 3 (`status, schema, value`), rewrite to `__SEC4_INTRINSIC_RES_OK__(...)`,
   - keep 2-arg `res.json(schema, value)` calls mapped to `sec4_rt_res_json(schema, value)`.
2. Added parser-safe marker rewrite helpers:
   - matching call-paren scanner with string/escape handling,
   - top-level argument counting with nested delimiter awareness.
3. Extended C backend coverage:
   - `compiler/sec4-core/tests/c_backend.rs` now asserts 3-arg `res.json` lowers to `sec4_rt_res_ok(...)`.
4. Extended CLI C emit integration coverage:
   - `compiler/sec4-cli/tests/json_output.rs` now includes and asserts status-form `res.json` C lowering.

## Why

`runtime/c/sec4_runtime.h` defines:

1. `sec4_rt_res_json(schema, value)` (2 args)
2. `sec4_rt_res_ok(status, schema, value)` (3 args)

Status-form `res.json` was previously lowered to `sec4_rt_res_json` with 3 args, causing deterministic clang compile failures for C emit paths.

## Validation

1. `cargo test -p sec4-core --test c_backend c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols`
2. `cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available`
3. `cargo run -q -p sec4 -- build --path benchmark-suite/services/sec4-lasm-workbench --emit c-bin`
