# 109 M7 Slice: res.text Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: adding plain-text response helper bridging for `res.text`.

## Plain-Text Response Intrinsic

### What it is

Added intrinsic support for:
- `res.text` / `res_text`

Lowered runtime ABI stub:
- `ailang_rt_res_text`

### Why it exists

The v0 stdlib surface includes `res.text(status, body)` as the simplest response constructor for non-JSON endpoints (for example `/health`). Without this bridge, the documented API shape cannot compile through `c-bin`.

### How it works internally

- Added semantic intrinsic entry in `semantic.rs`:
  - `res.text` requires `effects { net }`
  - return type is bootstrap `Unit` (consistent with other response emit helpers in M7)
- Added C rewrite mappings in `c_backend.rs`:
  - `res.text` / `res_text` -> `ailang_rt_res_text`
- Added runtime C declaration/definition in `runtime/c/`.
- Extended req/res bridge tests to assert `res.text` lowering in both core and CLI integration paths.

### Inputs, outputs, and constraints

- Input: `res.text(status, body)` calls in bootstrap service code.
- Output: generated C calls to `ailang_rt_res_text(status, body)`.
- Constraints:
  - runtime behavior remains a stub in M7 bootstrap mode.
  - strict response typing (`Response` object behavior) remains deferred to behavior-level runtime slices.

### Failure modes and diagnostics

- Missing `net` effect on functions that call `res.text` triggers effect diagnostics (`E4002` family).
- Misspelled intrinsic names fail semantic/C compile phases.

### Example usage

```ailang
fn health() effects { net } -> Int {
  res.text(200, 1);
  0
}
```

### Tradeoffs and next steps

- This closes a practical HTTP surface gap for simple endpoints with minimal complexity.
- Next steps are full response object semantics, status/body typing, and runtime behavior-level tests for real HTTP output.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - runtime ABI assertions include `ailang_rt_res_text`
  - updated req/res rewrite test to assert `ailang_rt_res_text(...)`
- `compiler/ailang-cli/tests/json_output.rs`
  - updated req/res `c-bin` integration fixture to call `res.text(200, ...)`
  - asserts generated C contains `ailang_rt_res_text(...)`
