# M39 - `c-bin` Canonical Runtime Source Only

## What changed

- Updated `compiler/sec4-cli/src/main.rs`:
  - `compile_c_binary(...)` now always resolves runtime assets from canonical repository path `runtime/c/sec4_runtime.{h,c}`.
  - Removed build-directory runtime-source copy fallback (`build/sec4_runtime.h`, `build/sec4_runtime.c`) from the compile path.
  - Missing canonical runtime files now fail deterministically with explicit source/header path diagnostics.
- Updated `compiler/sec4-cli/tests/json_output.rs`:
  - `build_emit_c_bin_compiles_binary_when_clang_available` now asserts canonical runtime files exist in `runtime/c/`.

## Why it exists

The build-folder runtime fallback introduced confusing operator behavior and ambiguous source-of-truth for the runtime ABI.  
`c-bin` now has one deterministic runtime source location.

## Behavior contract

- `sec4 build --emit c-bin` compiles generated C with:
  - `runtime/c/sec4_runtime.c`
  - include path `runtime/c/`
- If canonical runtime files are missing, build fails with status `2` and deterministic diagnostic text.
- Runtime ABI content in `build/` is no longer treated as a fallback source for compilation.
