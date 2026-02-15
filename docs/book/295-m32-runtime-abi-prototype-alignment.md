# 295 M32 Follow-up Slice: Runtime ABI Prototype Alignment

This chapter documents the second post-closure runtime stabilization slice.

## What it is

Updated:
- `runtime/c/sec4_runtime.h`
- `compiler/sec4-core/tests/c_backend.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key change:
- Runtime header prototypes for req/json/res intrinsics now match actual runtime definitions and generated call shapes:
  - `sec4_rt_req_json(int64_t schema)`
  - `sec4_rt_json_decode(int64_t ctx, int64_t schema, int64_t raw)`
  - `sec4_rt_json_encode(int64_t schema, int64_t value)`
  - `sec4_rt_res_json(int64_t schema, int64_t value)`
  - `sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value)`
  - `sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta)`

## Why it exists

The runtime previously declared several APIs with K&R-style no-prototype declarations in the header while definitions and generated C used explicit arguments. That weak ABI contract causes avoidable compiler warnings and makes runtime linkage behavior less predictable.

## How it works internally

1. `sec4_runtime.h` now exposes explicit argument lists for fixed-shape req/json/res intrinsics.
2. C backend runtime-asset tests were synchronized to assert the stricter header contract.
3. Existing runtime source definitions already matched these signatures, so no behavioral change was needed in implementation logic.

## Validation

Executed:
- `cargo test -p sec4-core --test c_backend`
- `cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available -- --nocapture`
- `cargo test -p sec4 --test json_output c_bin_runtime_gate_handles_are_non_stub_when_clang_available -- --nocapture`

## Trade-offs and next steps

- Trade-off:
  - Only fixed-shape req/json/res prototypes were hardened in this slice. Many flexible helper/sink wrappers still intentionally use legacy no-prototype declarations for bridge compatibility.
- Next:
  - Continue runtime de-stub with content-level validation paths (header CRLF/path normalization/URL policy checks) while preserving compatibility with current bridge call shapes.
