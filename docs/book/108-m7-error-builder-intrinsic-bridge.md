# 108 M7 Slice: Error Builder Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: bridging the standard error-model helper APIs from spec into the semantic layer, C lowering, and runtime ABI.

## Error Builder Intrinsics

### What it is

Added intrinsic support for:
- `err.validation` / `err_validation`
- `err.auth` / `err_auth`
- `err.notFound` / `err_not_found`
- `err.conflict` / `err_conflict`
- `err.rateLimit` / `err_rate_limit`
- `err.internal` / `err_internal`
- `err.withPath` / `err_with_path`
- `err.withDetail` / `err_with_detail`
- `err.withLimit` / `err_with_limit`
- `err.withDependency` / `err_with_dependency`
- `err.withCause` / `err_with_cause`

Lowered runtime ABI stubs:
- `ailang_rt_err_validation`
- `ailang_rt_err_auth`
- `ailang_rt_err_not_found`
- `ailang_rt_err_conflict`
- `ailang_rt_err_rate_limit`
- `ailang_rt_err_internal`
- `ailang_rt_err_with_path`
- `ailang_rt_err_with_detail`
- `ailang_rt_err_with_limit`
- `ailang_rt_err_with_dependency`
- `ailang_rt_err_with_cause`

### Why it exists

The v0 standard runtime error model defines explicit constructor and enrichment helpers. Without this bridge, spec-shaped service code that builds structured errors cannot compile through `c-bin`.

### How it works internally

- Added semantic intrinsic entries in `semantic.rs` for all `err.*` helpers.
- Each helper now returns `StdError` in bootstrap typing mode.
- Added C rewrite mappings in `c_backend.rs` from `err.*`/`err_*` calls to `ailang_rt_err_*` symbols.
- Added runtime declarations/definitions in `runtime/c/ailang_runtime.h` and `runtime/c/ailang_runtime.c`.
- Added C backend and CLI integration tests to assert rewriting and runnable `c-bin` output.

### Inputs, outputs, and constraints

- Input: error-helper calls in bootstrap projects.
- Output: generated C that links against runtime `ailang_rt_err_*` symbols.
- Constraints:
  - runtime implementations are still stubs in M7 bootstrap mode.
  - this slice validates compile/link coverage; full error-envelope behavior stays for behavior-level runtime slices.

### Failure modes and diagnostics

- Misspelled helper names can fail semantic or C compilation.
- Strict value-safety hardening for `err.withDetail` is covered in a later M7 slice (`152-m7-err-with-detail-value-safety-hardening.md`).

### Example usage

```ailang
fn main() -> Int {
  let base = err.validation("VALIDATION.BAD_REQUEST", "invalid input");
  let internal = err.internal("internal");
  err.withDetail(base, "field", 2);
  err.withCause(base, internal);
  0
}
```

### Tradeoffs and next steps

- This keeps runtime bridge coverage aligned with the documented error API surface without overcommitting behavior too early.
- Next steps are strict typed contracts for error payload fields and full response-envelope wiring for `HttpError` return paths.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - added `c_backend_rewrites_error_builder_intrinsics_to_runtime_symbols`
  - runtime ABI assertions include all `ailang_rt_err_*` symbols
- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_error_builder_intrinsics_when_clang_available`
