# 93 M7 Bootstrap: HTTP Router Intrinsic Runtime Bridge

This chapter starts M7 with a minimal runtime bridge for router-level HTTP intrinsics.

## HTTP Router Intrinsic Lowering

### What it is

C emission now rewrites:
- `http.router(...)`
- `http.get(...)`
- `http.post(...)`
- `http.serve(...)`

to runtime symbols:
- `ailang_rt_http_router(...)`
- `ailang_rt_http_route_get(...)`
- `ailang_rt_http_route_post(...)`
- `ailang_rt_http_serve(...)`

Runtime ABI stubs for these symbols were added under `runtime/c/`.

### Why it exists

M7 requires a router/runtime bridge. This slice provides the first executable bridge seam so router-shaped programs pass semantic checks and compile through `c-bin` while real server behavior is implemented in later slices.

### How it works internally

- Semantic layer now recognizes:
  - `http.router`, `http.get`, `http.post` (no effects)
  - `http.serve` (`net` effect required)
- Intrinsic namespace support now includes `http`.
- C emitter rewrite table maps HTTP router intrinsics to runtime ABI symbols.
- Runtime ABI (`runtime/c/ailang_runtime.h/.c`) now declares/defines corresponding stub functions.

### Inputs, outputs, and constraints

- Input: AILang code using router intrinsics.
- Output: C-valid runtime calls and runnable `c-bin` artifacts.
- Constraints:
  - runtime behavior is still placeholder/stub (returns numeric placeholders).
  - `http.serve` must still satisfy effect declaration checks (`effects { net }`).
  - current semantic model does not use function names as first-class values in these fixture shapes; tests use literal placeholders for handler arguments.

### Failure modes and diagnostics

- Missing `effects { net }` when calling `http.serve` fails semantic checks (`E4002` effect-used-not-declared).
- Unknown/inconsistent intrinsic spellings remain unlowered and may fail C compile/link.

### Example usage

```ailang
fn buildRouter() effects { net } -> Int {
  let router = http.router();
  http.get(router, 1, 1);
  http.post(router, 1, 1);
  http.serve(1, router);
  0
}
```

### Tradeoffs and next steps

- This is bridge scaffolding, not real HTTP serving.
- Route registration and serving semantics are intentionally deferred to later M7 runtime slices.
- Next step: begin replacing router stubs with concrete runtime data structures and request loop behavior.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - new `c_backend_rewrites_http_router_intrinsics_to_runtime_symbols`
- `compiler/ailang-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_http_router_intrinsics_when_clang_available`
