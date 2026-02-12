# 100 M7 Slice: Request Source Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: bridging request-source extraction intrinsics through the C runtime path.

## Request Source Intrinsic Coverage

### What it is

Added runtime bridge coverage for request-source intrinsics:
- `req.body` / `req_body`
- `req.query` / `req_query`
- `req.pathParam` / `req_path_param`
- `req.header` / `req_header`

They now lower to runtime ABI stubs:
- `ailang_rt_req_body`
- `ailang_rt_req_query`
- `ailang_rt_req_path_param`
- `ailang_rt_req_header`

### Why it exists

M7’s HTTP runtime surface includes request extraction beyond JSON decode. Before this slice, only `req.json` had compile-path runtime lowering, so projects using other request sources could pass semantic checks but fail at generated C compilation.

### How it works internally

- Extended intrinsic rewrite table in `c_backend.rs` with dotted and underscore aliases for all four request-source calls.
- Added runtime C declarations/definitions in `runtime/c/ailang_runtime.h` and `runtime/c/ailang_runtime.c`.
- Extended core C-backend tests and CLI `c-bin` integration coverage to assert the generated C contains expected runtime symbols.
- Updated `examples/hello-api/src/main.ai` to exercise request-source calls in the sample service flow.

### Inputs, outputs, and constraints

- Input: AILang code using request extraction helpers.
- Output: successful `build --emit c-bin` with request-source calls lowered to runtime ABI stubs.
- Constraints:
  - runtime return values are still placeholder stubs.
  - this slice validates bridge completeness, not full runtime request parsing behavior.

### Failure modes and diagnostics

- Misspelled helper names remain unresolved and can fail C compilation.
- Missing `net` effect declarations still trigger semantic diagnostics for these intrinsics.

### Example usage

```ailang
fn createUser(schema: Schema<Int>) effects { net } -> Int {
  req.body(1, 2);
  req.query(1, 2);
  req.pathParam(1, 2);
  req.header(1, 2);
  req.json(schema);
  0
}
```

### Tradeoffs and next steps

- This gives full request-source API-shape coverage for bootstrap compilation while runtime semantics stay stubbed.
- Next step is replacing placeholder stubs with real request object extraction, typed decoding, and budget enforcement behavior.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - req/res rewrite test now covers request-source intrinsics
  - runtime header/source assertions include new request-source ABI symbols
- `compiler/ailang-cli/tests/json_output.rs`
  - req/res `c-bin` integration fixture now asserts request-source runtime symbol lowering
