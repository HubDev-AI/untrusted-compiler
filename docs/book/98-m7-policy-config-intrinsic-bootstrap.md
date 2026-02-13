# 98 M7 Slice: Policy-Config Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: compiling policy-config helper intrinsics through the runtime bridge.

## Policy-Config Helper Intrinsics

### What it is

Added semantic recognition and C-runtime lowering for:
- `sec.defaultHeaders` / `sec_default_headers`
- `cors.fromPolicy` / `cors_from_policy`
- `csrf.fromPolicy` / `csrf_from_policy`
- `auth.fromPolicy` / `auth_from_policy`

They are lowered to runtime ABI stubs:
- `sec4_rt_sec_default_headers`
- `sec4_rt_cors_from_policy`
- `sec4_rt_csrf_from_policy`
- `sec4_rt_auth_from_policy`

### Why it exists

The security-first router bootstrap spec uses policy-derived helpers as the canonical configuration path. Without these bridges, middleware examples compile only with manual placeholder arguments and drift from the intended API shape.

### How it works internally

- Extended semantic intrinsic registry in `semantic.rs` so policy-config helper names are known callable intrinsics.
- Extended C expression intrinsic rewrite rules in `c_backend.rs` to map helper calls to runtime ABI symbols.
- Added runtime C declarations/definitions in `runtime/c/sec4_runtime.h` and `runtime/c/sec4_runtime.c`.
- Updated `examples/hello-api/src/main.ut` to include these policy-config helper calls in the bootstrap flow.

### Inputs, outputs, and constraints

- Input: Untrusted<T> code invoking policy-config helpers.
- Output: generated C calling runtime helper stubs and successful `c-bin` compilation.
- Constraints:
  - helper behavior is currently placeholder runtime stubs.
  - this slice validates compile-path/API-shape alignment, not final policy semantics.

### Failure modes and diagnostics

- Misspelled helper names can remain unresolved and fail semantic or C compile checks.
- Runtime behavior-level mismatches remain deferred to later M7/M8 security runtime slices.

### Example usage

```ut
fn main() -> Int {
  sec.defaultHeaders();
  cors.fromPolicy();
  csrf.fromPolicy();
  auth.fromPolicy();
  sec.withSecurityHeaders();
  cors.withCors();
  csrf.withCsrf();
  auth.withAuth();
  0
}
```

### Tradeoffs and next steps

- This keeps momentum on the canonical secure bootstrap surface without blocking on full runtime middleware logic.
- Next steps are to replace these stubs with policy-backed runtime config generation and enforce behavior-level constraints in security middleware paths.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - added `c_backend_rewrites_policy_config_intrinsics_to_runtime_symbols`
- `compiler/sec4-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_policy_config_intrinsics_when_clang_available`
