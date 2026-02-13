# 79 M6 Slice: Clang Compile/Link Pipeline

This chapter documents the next M6 vertical slice: compiling emitted C into a runnable binary from the CLI.

Follow-up run-command integration is documented in `docs/book/80-m6-run-command-via-c-bin.md`.

## Scope delivered
- Added CLI emit target:
  - `sec4 build --emit c-bin`
- `c-bin` pipeline now:
  1. lowers AST -> MIR
  2. emits C source
  3. writes `build/generated.c`
  4. invokes `clang` to produce `build/<package-name>`
- Added integration coverage for the compile/link path (clang-gated test).

## What changed
- `BuildEmitTarget` now includes `CBin`.
- CLI build command now has `compile_c_binary(...)` helper:
  - creates build dir,
  - writes generated C,
  - invokes `clang -std=c11 -O2 -o <binary>`.
- Emitted C backend now forces `main` to return `int` for clang compatibility.

## Why this matters
- This is the first end-to-end runnable backend flow for Untrusted<T>.
- The project now has a concrete path from source to executable artifact.
- It validates the M5 MIR work against a real native toolchain.

## Tests added/updated
- CLI integration test:
  - `build_emit_c_bin_compiles_binary_when_clang_available`
- Existing C emitter tests updated to match C `main` signature requirements.

## Tradeoffs
- The compile pipeline currently depends on host `clang` availability.
- Runtime/ABI surface is still minimal and currently targets the existing reduced language subset.
- C emission remains intentionally simple; richer ABI/runtime integration is still pending.

## Next step
- Wire `sec4 run` to execute binaries produced by the C compile pipeline.
