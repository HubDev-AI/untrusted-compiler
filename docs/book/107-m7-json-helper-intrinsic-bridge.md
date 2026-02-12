# 107 M7 Slice: JSON Helper Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: adding JSON helper intrinsic bridge coverage for `json.decode` and `json.encode`.

## JSON Helper Intrinsics

### What it is

Added intrinsic support for:
- `json.decode` / `json_decode`
- `json.encode` / `json_encode`

Lowered runtime ABI stubs:
- `ailang_rt_json_decode`
- `ailang_rt_json_encode`

Also added `Json` as a recognized primitive type name in semantic analysis.

### Why it exists

The v0 stdlib surface includes explicit JSON helper APIs alongside req/res helpers. Without this bridge, spec-shaped code that uses `json.decode` / `json.encode` cannot compile through `c-bin`.

### How it works internally

- Added semantic intrinsic entries in `semantic.rs`:
  - `json.decode` returns `Unknown` (schema-typed decode result placeholder in bootstrap mode)
  - `json.encode` returns `Json`
- Added `json` intrinsic namespace recognition.
- Added C rewrite mappings in `c_backend.rs`.
- Added runtime C declarations/definitions in `runtime/c/`.
- Added `security_map` tags:
  - `json.decode` -> `gate.schema.json_decode`
  - `json.encode` -> `sink.json.encode`
- Added core and CLI integration tests for symbol lowering and runnable `c-bin` output.

### Inputs, outputs, and constraints

- Input: helper calls in bootstrap projects.
- Output: generated C with runtime JSON helper symbol calls.
- Constraints:
  - runtime JSON semantics are still placeholder stubs in M7 bootstrap mode.
  - this slice validates compile-path coverage and metadata tagging.

### Failure modes and diagnostics

- Misspelled helper names can fail semantic/C compile phases.
- Full schema/value enforcement for generic `json.decode/encode` calls is intentionally deferred to behavior-level runtime/type slices.

### Example usage

```ailang
fn main() -> Int {
  json.decode(1, 2, 3);
  json.encode(2, 3);
  0
}
```

### Tradeoffs and next steps

- This closes a bootstrap API gap without overfitting runtime behavior too early.
- Next steps are stronger schema-aware typing and runtime decode/encode behavior aligned with strict policy modes.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - added `c_backend_rewrites_json_helper_intrinsics_to_runtime_symbols`
  - runtime ABI assertions include JSON helper symbols
- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_json_helper_intrinsics_when_clang_available`
