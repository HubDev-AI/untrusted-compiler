# 81 M6 Slice: Runtime ABI Scaffold and Link Integration

This chapter documents the next M6 vertical slice: introducing explicit runtime ABI artifacts and wiring them into the C compile/link flow.

## Runtime ABI Scaffold

### What it is

Untrusted<T> now emits runtime ABI companion files for the C backend:
- `sec4_runtime.h`
- `sec4_runtime.c`

Generated C now includes the runtime header explicitly.

### Why it exists

The C backend needs a stable integration seam for runtime intrinsics and stdlib-backed behavior. Adding explicit runtime files now makes that seam concrete before larger runtime features land.

### How it works internally

- `sec4-core` now exposes:
  - `emit_runtime_header()`
  - `emit_runtime_source()`
- `emit_c_program(...)` now emits:
  - `#include "sec4_runtime.h"`
- CLI compile flow (`build --emit c-bin`) now:
  1. writes `build/generated.c`
  2. writes `build/sec4_runtime.h`
  3. writes `build/sec4_runtime.c`
  4. invokes `clang` with both C files and `-I build` include path

The initial runtime functions are identity stubs (`sec4_rt_identity_i64`, `sec4_rt_identity_bool`) to lock the ABI flow without adding behavior-heavy runtime logic yet.

### Inputs, outputs, and constraints

- Input: lowered MIR program from the existing frontend/lowering pipeline.
- Outputs in `build/`:
  - generated C translation unit
  - runtime header/source translation units
  - compiled binary (`c-bin` target)
- Constraint: host must have `clang` available for compile/link steps.

### Failure modes and diagnostics

- Runtime artifact write failures produce explicit CLI errors:
  - `could not write runtime header ...`
  - `could not write runtime source ...`
- Missing/failed clang invocation remains surfaced via existing compile pipeline diagnostics.

### Example usage

```bash
cargo run -p sec4 -- build --emit c-bin --path examples/hello
```

Expected artifacts:
- `examples/hello/build/generated.c`
- `examples/hello/build/sec4_runtime.h`
- `examples/hello/build/sec4_runtime.c`
- `examples/hello/build/hello`

### Tradeoffs and next steps

- Runtime ABI surface is intentionally minimal in this slice (identity helpers only).
- No effectful runtime intrinsics are wired yet; this is scaffolding for the next runtime-heavy slices.
- Next step: expand runtime ABI with concrete stdlib/runtime primitives and call lowering hooks.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - asserts generated C includes runtime header
  - validates emitted runtime header/source content
- `compiler/sec4-cli/tests/json_output.rs`
  - `build_emit_c_bin_compiles_binary_when_clang_available` now asserts runtime files are written
