# 102 M7 Slice: CORS Origin Gate Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: adding compile-path support for the typed CORS origin gate helper.

## `cors.origin` Gate Intrinsic

### What it is

Added intrinsic support for:
- `cors.origin` / `cors_origin`

It lowers to runtime ABI stub:
- `ailang_rt_cors_origin`

### Why it exists

The CORS spec in AILang docs includes typed origin validation. Without bridge support, CORS gate usage could not be represented end-to-end in `c-bin` flows.

### How it works internally

- Added semantic intrinsic entry for `cors.origin` returning `Origin`.
- Classified `cors.origin` as an untrusted-string gate so semantic gate checks require `Untrusted<String>` input.
- Added C backend rewrite mapping to `ailang_rt_cors_origin`.
- Added runtime C declaration/definition in `runtime/c/`.
- Added security-map tags so this call is visible as `gate.cors.origin` in security metadata.
- Added `c-bin` integration coverage and exercised usage in `examples/hello-api`.

### Inputs, outputs, and constraints

- Input: untrusted origin string values (for example from `req.header`).
- Output: typed `Origin` value (bootstrap runtime currently returns placeholder stub values).
- Constraints:
  - runtime validation semantics are still placeholder in M7 bootstrap mode.
  - this slice ensures compile-path and typing alignment, not final CORS runtime behavior.

### Failure modes and diagnostics

- Passing trusted non-untrusted-string values to `cors.origin` now fails trust-gate checks.
- Misspelled helper names remain unresolved and fail semantic/C compilation.

### Example usage

```ailang
fn main() effects { net } -> Int {
  let origin = req.header(1, 2);
  cors.origin(origin);
  0
}
```

### Tradeoffs and next steps

- This keeps CORS typed-gate APIs available early for language/runtime iteration.
- Next steps are real origin parsing/validation semantics and full middleware preflight behavior in later M7/M8 runtime slices.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - added `c_backend_rewrites_cors_origin_intrinsic_to_runtime_symbol`
  - runtime ABI symbol assertions now include `ailang_rt_cors_origin`
- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_cors_origin_intrinsic_when_clang_available`
- `compiler/ailang-core/src/security_map.rs`
  - added gate tag mapping for `cors.origin` / `cors_origin`
