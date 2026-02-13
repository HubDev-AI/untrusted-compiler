# 88 M6 Slice: `req`/`res` Intrinsic Runtime Lowering

This chapter documents the next M6 vertical slice: lowering core request/response intrinsic calls to runtime ABI symbols so request/response-shaped code paths can compile via `c-bin`.

## Request/Response Intrinsic Rewriting

### What it is

C emission now rewrites these intrinsic calls:
- `req.json(...)`
- `res.json(...)`
- `res.html(...)`

and underscore aliases to runtime symbols:
- `sec4_rt_req_json(...)`
- `sec4_rt_res_json(...)`
- `sec4_rt_res_html(...)`

Runtime ABI now includes corresponding stub declarations/definitions.

### Why it exists

As with other dotted intrinsics, direct `req.*`/`res.*` names are not valid C symbols. This slice extends ABI lowering so early HTTP/JSON-style code can pass through the compiler backend and native build pipeline while runtime behavior is still stubbed.

### How it works internally

- Extended `lower_c_expr(...)` replacements for `req.json`, `res.json`, and `res.html`.
- Added runtime stubs in `runtime/c/sec4_runtime.h` and `runtime/c/sec4_runtime.c`.
- Existing emission paths already route every expression through `lower_c_expr(...)`, so rewrites apply consistently in statements and return/branch contexts.

### Inputs, outputs, and constraints

- Input: MIR expression text containing supported req/res intrinsic calls.
- Output: C-valid runtime call symbols and successful compile/link flow.
- Constraints:
  - runtime functions are placeholders returning stub values.
  - semantic checks still enforce strict schema argument constraints for `res.json`.

### Failure modes and diagnostics

- Invalid `res.json` schema arguments are still rejected before backend (`E4004`).
- Missing net effect declarations for req/res calls are still rejected in semantic analysis (`E4002`).
- Unsupported intrinsic spellings remain unlowered and may fail in C compile/link.

### Example usage

```ut
fn decode(schema: Schema<Int>) effects { net } -> Int {
  req.json(schema);
  0
}

fn encode(schema: Schema<Int>) effects { net } -> Int {
  res.json(schema, 1);
  res.html(1);
  0
}
```

### Tradeoffs and next steps

- Behavior is compile-path scaffolding, not full runtime HTTP/JSON semantics.
- Rewriting remains string-based in M6.
- Next step: introduce real runtime request/response data types and behavior in M7 while preserving these call-lowering hooks.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - new `c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols`
- `compiler/sec4-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_req_res_intrinsics_when_clang_available`
