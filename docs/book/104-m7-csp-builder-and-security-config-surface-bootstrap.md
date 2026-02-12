# 104 M7 Slice: CSP Builder and Security Config Surface Bootstrap

This chapter documents the next M7 bootstrap slice: adding compile-path support for CSP builder helpers and security-config type names.

## CSP Builder Intrinsics + Config Surface Types

### What it is

Added intrinsic support for:
- `sec.csp` / `sec_csp`
- `sec.cspAdd` / `sec_csp_add`

Lowered runtime ABI stubs:
- `ailang_rt_sec_csp`
- `ailang_rt_sec_csp_add`

Added semantic primitive type-name support for security config shapes:
- `CorsConfig`, `SecurityHeadersConfig`, `CsrfConfig`, `AuthConfig`
- `CspConfig`, `CspPolicy`
- `CorsOrigins`, `OriginPattern`
- `Principal`, `Caps`

### Why it exists

The security middleware spec uses structured config types and CSP builders. Without these names and bridge entrypoints, spec-shaped service code compiles only with placeholders and drifts from the intended language surface.

### How it works internally

- Extended primitive type catalog in `semantic.rs` with security-config surface names.
- Added intrinsic specs for `sec.csp` and `sec.cspAdd` returning `CspPolicy`.
- Added C intrinsic rewrite mappings in `c_backend.rs`.
- Added runtime C declarations/definitions in `runtime/c/`.
- Extended policy-config integration fixture to assert CSP builder symbol lowering.
- Added CLI integration coverage proving security config type names compile through `c-bin`.
- Updated `examples/hello-api` bootstrap to call CSP builder helpers.

### Inputs, outputs, and constraints

- Input: security-config typed signatures and CSP helper calls.
- Output: compile-path success and generated C symbols for CSP helpers.
- Constraints:
  - runtime CSP semantics remain placeholder stubs.
  - this slice validates API shape and bridge completeness, not final header/CSP enforcement.

### Failure modes and diagnostics

- Unknown-type errors for listed security config names are removed.
- Misspelled CSP helper names can still fail semantic/C compile stages.

### Example usage

```ailang
fn main() -> Int {
  let csp = sec.csp();
  sec.cspAdd(csp, 1, 2);
  0
}
```

### Tradeoffs and next steps

- This unblocks spec-aligned coding patterns while runtime behavior remains incremental.
- Next step is replacing CSP/runtime stubs with typed policy-driven header emission behavior.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - policy-config rewrite coverage now asserts `sec.csp` and `sec.cspAdd` lowering
  - runtime ABI assertions include CSP helper symbols
- `compiler/ailang-cli/tests/json_output.rs`
  - policy-config `c-bin` fixture now asserts CSP helper symbol lowering
  - added `build_emit_c_bin_accepts_security_config_surface_types_when_clang_available`
