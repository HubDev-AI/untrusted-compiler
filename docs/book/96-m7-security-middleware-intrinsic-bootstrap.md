# 96 M7 Slice: Security Middleware Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: making security middleware calls compile through the runtime bridge.

## Middleware Intrinsic Rewriting

### What it is

Semantic and C backend now support these middleware intrinsics:
- `withSecurityHeaders` / `sec.withSecurityHeaders`
- `withCors` / `cors.withCors`
- `withCsrf` / `csrf.withCsrf`
- `withAuth` / `auth.withAuth`

They are lowered to runtime ABI stubs:
- `sec4_rt_with_security_headers`
- `sec4_rt_with_cors`
- `sec4_rt_with_csrf`
- `sec4_rt_with_auth`

### Why it exists

Security-first router bootstrap is a core Untrusted<T> goal. This slice ensures middleware-shaped code paths are executable in the current bridge architecture while concrete middleware behavior is implemented later.

### How it works internally

- Semantic intrinsic registry now recognizes middleware names (bare, underscore, and dotted forms).
- Intrinsic namespace support now includes `sec`, `cors`, `csrf`, and `auth`.
- C emitter rewrite table maps middleware calls to runtime ABI symbols.
- Runtime C ABI (`runtime/c/`) now includes corresponding stub functions.
- Rewrite ordering for dotted and bare aliases was adjusted so dotted forms are rewritten first, preventing partial replacement corruption.

### Inputs, outputs, and constraints

- Input: Untrusted<T> code invoking middleware intrinsics.
- Output: C-valid runtime stub calls and successful `c-bin` compilation.
- Constraints:
  - runtime middleware behavior is currently placeholder.
  - this slice validates compile-path bridging, not HTTP/security semantics.

### Failure modes and diagnostics

- Incorrect intrinsic spellings can remain unlowered and fail C compilation.
- Future behavior-level middleware checks are still pending in M7 runtime work.

### Example usage

```ut
fn main() -> Int {
  sec.withSecurityHeaders();
  cors.withCors();
  csrf.withCsrf();
  auth.withAuth();
  0
}
```

### Tradeoffs and next steps

- Current stubs keep progress incremental but do not enforce runtime header/CORS/CSRF/auth behavior yet.
- This closes the compile-path bridge for middleware bootstrap shape.
- Next step: implement actual middleware runtime semantics and policy-driven defaults while preserving these entrypoints.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - new `c_backend_rewrites_security_middleware_intrinsics_to_runtime_symbols`
- `compiler/sec4-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_security_middleware_intrinsics_when_clang_available`
