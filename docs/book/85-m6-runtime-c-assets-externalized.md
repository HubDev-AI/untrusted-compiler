# 85 M6 Slice: Runtime C Assets Externalized

This chapter documents the next M6 vertical slice: moving runtime ABI C code out of Rust string literals into canonical runtime source files.

## External Runtime Asset Wiring

### What it is

Runtime ABI artifacts now live in dedicated source files:
- `runtime/c/ailang_runtime.h`
- `runtime/c/ailang_runtime.c`

`ailang-core` now emits runtime header/source content via `include_str!` from these files.

### Why it exists

Keeping runtime C assets in real files improves maintainability and makes runtime evolution (M6+ and M7 runtime work) clearer than editing embedded multiline Rust strings.

### How it works internally

- Added runtime source files under `runtime/c/`.
- Updated:
  - `emit_runtime_header()` -> `include_str!("../../../runtime/c/ailang_runtime.h")`
  - `emit_runtime_source()` -> `include_str!("../../../runtime/c/ailang_runtime.c")`
- Existing CLI compile flow remains unchanged:
  - writes runtime assets into project `build/`
  - compiles generated C + runtime C with `clang`

### Inputs, outputs, and constraints

- Input: canonical runtime ABI files in repository runtime directory.
- Output: same emitted runtime header/source payload as before, but sourced from external files.
- Constraint: relative include path from `ailang-core/src/c_backend.rs` must remain valid if project layout changes.

### Failure modes and diagnostics

- If runtime files are moved/renamed without updating `include_str!` paths, compile-time errors occur in `ailang-core`.
- Runtime API mismatches continue to surface in existing C emitter and CLI compile tests.

### Example usage

No CLI behavior change for users:

```bash
cargo run -p ailang -- build --emit c-bin --path examples/hello
```

Runtime assets written to the target project build dir still include:
- `build/ailang_runtime.h`
- `build/ailang_runtime.c`

### Tradeoffs and next steps

- This adds an extra repository surface (`runtime/c`), but makes runtime development explicit.
- Runtime C code is still minimal stubs in M6.
- Next step: grow runtime ABI functions in `runtime/c` as M7 runtime/HTTP/JSON primitives land.

## Tests re-validated

- `compiler/ailang-core/tests/c_backend.rs`
- `compiler/ailang-cli/tests/json_output.rs` (`build_emit_c_bin*` coverage)
