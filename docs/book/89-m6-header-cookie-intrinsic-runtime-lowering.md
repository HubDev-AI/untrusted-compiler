# 89 M6 Slice: Header/Cookie Intrinsic Runtime Lowering

This chapter documents the next M6 vertical slice: lowering response header/cookie intrinsics to runtime ABI symbols.

## Header/Cookie Intrinsic Rewriting

### What it is

C emission now rewrites:
- `res.setHeader(...)`
- `res.addCookie(...)`

and underscore aliases to runtime symbols:
- `ailang_rt_set_header(...)`
- `ailang_rt_set_cookie(...)`

Runtime ABI now includes stub declarations/definitions for both functions.

### Why it exists

Typed header/cookie APIs are core to the security model. This slice keeps backend progress aligned by ensuring these intrinsic call paths compile through the C backend while full runtime HTTP behavior is still being built.

### How it works internally

- Added rewrite rules in `lower_c_expr(...)` for dotted and underscore forms.
- Added runtime stubs in `runtime/c/ailang_runtime.h` and `runtime/c/ailang_runtime.c`.
- Existing expression rewrite flow applies these mappings across emitted statements/returns/branches.

### Inputs, outputs, and constraints

- Input: MIR expression text with supported header/cookie intrinsic spellings.
- Output: C-valid runtime calls (`ailang_rt_set_header`, `ailang_rt_set_cookie`).
- Constraints:
  - stubs currently return placeholder `int64_t` values.
  - this is compile-path scaffolding; not full HTTP header/cookie semantics.

### Failure modes and diagnostics

- Missing net effects for these intrinsics are still rejected semantically (`E4002`).
- Unsupported intrinsic naming forms remain unlowered and may fail C compilation.

### Example usage

```ailang
fn configure() effects { net } -> Int {
  res.setHeader(1, 2);
  res.addCookie(1);
  0
}
```

### Tradeoffs and next steps

- Runtime behavior is intentionally stubbed in M6 to prioritize backend plumbing.
- Rewrites are still string-based.
- Next step: connect these ABI hooks to real typed header/cookie runtime behavior in M7 security middleware work.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - new `c_backend_rewrites_header_and_cookie_intrinsics_to_runtime_symbols`
- `compiler/ailang-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_header_cookie_intrinsics_when_clang_available`
