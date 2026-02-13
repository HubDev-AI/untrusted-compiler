# 99 M7 Slice: Success Envelope Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: compiling standard success-envelope helper calls through the runtime bridge.

## Success Envelope Helper Intrinsics

### What it is

Added semantic recognition and C-runtime lowering for:
- `res.ok` / `res_ok`
- `res.okMeta` / `res_ok_meta`

They lower to runtime ABI stubs:
- `ailang_rt_res_ok`
- `ailang_rt_res_ok_meta`

### Why it exists

The v0 response surface includes optional success envelopes. This slice keeps the documented API executable during bootstrap so examples can use `res.ok`-style responses without waiting for full runtime envelope semantics.

### How it works internally

- Added intrinsic specs in `semantic.rs`:
  - both helpers are treated as `net` effect sinks returning `Unit`.
- Extended strict schema checks (`E4004`) to validate helper signatures:
  - `res.ok(status, schema, value)`
  - `res.okMeta(status, schema, value, meta)`
- Extended C intrinsic rewriting in `c_backend.rs` to map calls to runtime ABI symbols.
- Added runtime C declarations/definitions in `runtime/c/ailang_runtime.h` and `runtime/c/ailang_runtime.c`.
- Extended security-map sink tagging so `sec.audit` treats these helpers as JSON response sinks.

### Inputs, outputs, and constraints

- Input: AILang code calling success-envelope helpers.
- Output: generated C invoking runtime stubs and successful `c-bin` builds.
- Constraints:
  - runtime envelope structure/serialization behavior is still placeholder in M7 bootstrap mode.
  - this slice focuses on compile-path and policy/sink alignment, not final HTTP payload semantics.

### Failure modes and diagnostics

- Invalid helper arity or non-numeric status yields `E4004` under strict schema mode.
- Invalid schema/value pairings still produce schema mismatch diagnostics.
- Schema descriptors are narrowed in a later slice to `String` or `Schema<_>` (`177-m7-json-response-schema-descriptor-narrowing.md`).
- Meta payload secret/taint rejection for `res.okMeta` is hardened in a later slice (`178-m7-json-response-meta-secret-taint-hardening.md`).
- Misspelled helper names can remain unresolved and fail semantic/C compile stages.

### Example usage

```ailang
fn createUser(schema: Schema<Int>) effects { net } -> Int {
  res.ok(201, schema, 1);
  res.okMeta(201, schema, 1, 2);
  0
}
```

### Tradeoffs and next steps

- This keeps the documented response API available early while runtime behavior remains stubbed.
- Next steps are to wire real envelope encoding behavior and policy-controlled envelope toggles in runtime-level M7/M8 slices.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - extended req/res rewrite coverage with `res.ok` and `res.okMeta`
  - runtime ABI header/source assertions include new stub symbols
- `compiler/ailang-cli/tests/json_output.rs`
  - extended req/res `c-bin` integration fixture to assert `res.ok` and `res.okMeta` lowering
- `compiler/ailang-core/tests/security_map.rs`
  - existing suite validates updated JSON sink tag mapping remains consistent
