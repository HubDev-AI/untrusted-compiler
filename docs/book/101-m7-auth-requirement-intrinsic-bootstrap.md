# 101 M7 Slice: Auth Requirement Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: compiling auth requirement helper calls through the runtime bridge.

## Auth Requirement Helper Intrinsics

### What it is

Added intrinsic support for:
- `auth.require` / `auth_require`
- `auth.requireRole` / `auth_require_role`

These now lower to runtime ABI stubs:
- `sec4_rt_auth_require`
- `sec4_rt_auth_require_role`

### Why it exists

The v0 auth middleware API includes handler-side principal checks (`require` / `requireRole`). Without bridge support, examples or services that use these helpers could pass parse-level checks but fail in generated C output.

### How it works internally

- Added semantic intrinsic registry entries in `semantic.rs`.
- Added C lowering replacements in `c_backend.rs` for dotted and underscore aliases.
- Added runtime C declarations/definitions in `runtime/c/sec4_runtime.h` and `runtime/c/sec4_runtime.c`.
- Added core and CLI integration tests for end-to-end `c-bin` lowering.
- Updated `examples/hello-api/src/main.ut` to include auth requirement helper calls in handler flow.

### Inputs, outputs, and constraints

- Input: Untrusted<T> code invoking auth requirement helpers.
- Output: successful `build --emit c-bin` with calls lowered to runtime ABI symbols.
- Constraints:
  - runtime auth semantics are still placeholder stubs in bootstrap mode.
  - this slice validates compile-path and API-shape continuity only.

### Failure modes and diagnostics

- Misspelled helper names can remain unresolved and fail C generation/compilation.
- Behavior-level auth checks (actual principal extraction/role matching) remain future runtime work.

### Example usage

```ut
fn createUser(schema: Schema<Int>) effects { net } -> Int {
  auth.require(1);
  auth.requireRole(1, 2);
  req.json(schema);
  0
}
```

### Tradeoffs and next steps

- This keeps documented auth helper APIs usable early while preserving incremental runtime delivery.
- Next step is wiring real principal context and role enforcement behavior behind these entrypoints.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - added `c_backend_rewrites_auth_requirement_intrinsics_to_runtime_symbols`
  - runtime ABI symbol assertions include auth requirement helpers
- `compiler/sec4-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_auth_requirement_intrinsics_when_clang_available`
