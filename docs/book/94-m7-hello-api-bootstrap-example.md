# 94 M7 Slice: `hello-api` Bootstrap Example

This chapter documents the next M7 bootstrap slice: adding a concrete HTTP/JSON-shaped example project in `examples/`.

## Example Project: `examples/hello-api`

### What it is

Added a new example project:
- `examples/hello-api/sec4.toml`
- `examples/hello-api/src/main.ut`
- `examples/hello-api/build/.gitignore`

It demonstrates:
- security middleware bootstrap calls (`sec.withSecurityHeaders`, `cors.withCors`, `csrf.withCsrf`, `auth.withAuth`)
- router bootstrap calls (`http.router/get/post/serve`)
- request decode call shape (`req.json`)
- response encode/render call shapes (`res.json`, `res.html`)

### Why it exists

M7 needs a sample service path. This slice introduces a stable example target for iterative runtime work and tests, even before real HTTP server behavior is implemented.

### How it works internally

- Example code is intentionally compile-oriented and stub-friendly.
- Runtime behavior is currently provided by M6/M7 bridge stubs.
- Added a CLI integration test:
  - `build_emit_c_bin_compiles_hello_api_example_when_clang_available`
  - verifies `hello-api` compiles via `c-bin`
  - checks generated C includes expected lowered runtime calls.

### Inputs, outputs, and constraints

- Input: `examples/hello-api/src/main.ut`.
- Output on build:
  - `examples/hello-api/build/generated.c`
  - `examples/hello-api/build/hello-api` (compiled binary)
- Constraints:
  - behavior remains stubbed (no real request loop yet).
  - `http.serve` and req/res intrinsics still require semantic effect declarations.

### Failure modes and diagnostics

- Missing required effects in the example would fail semantic checks (`E4002`).
- Intrinsic-lowering regressions would be caught by generated-C assertions in the integration test.
- Missing clang skips the integration test.

### Example usage

```bash
cargo run -p sec4 -- build --path examples/hello-api --emit c-bin
```

### Tradeoffs and next steps

- This is a structural example, not a real network service yet.
- It gives M7 a stable target for evolving runtime behavior without rewriting fixtures each slice.
- Next step: replace HTTP/JSON stubs with concrete runtime request/response handling while preserving this example contract.

## Tests updated

- `compiler/sec4-cli/tests/json_output.rs`
  - added `build_emit_c_bin_compiles_hello_api_example_when_clang_available`
