# 78 M6 Bootstrap: C Emitter from MIR

This chapter documents the first M6 vertical slice: emitting C source from MIR through the AILang CLI.

Follow-up clang compile/link integration is documented in `docs/book/79-m6-clang-compile-link-pipeline.md`.

## Scope delivered
- Added a C backend emitter in `ailang-core`:
  - `emit_c_program(&MirProgram) -> String`
- Added CLI support:
  - `ailang build --emit c`
- Added test coverage:
  - core C backend tests in `compiler/ailang-core/tests/c_backend.rs`
  - CLI integration coverage in `compiler/ailang-cli/tests/json_output.rs`

## What changed
- New module: `compiler/ailang-core/src/c_backend.rs`.
- `ailang-core` now exports the C emitter via `emit_c_program`.
- CLI build emit targets now include `c` and print generated C from lowered MIR.

## C emission shape (current)
- Emits standard headers:
  - `#include <stdbool.h>`
  - `#include <stdint.h>`
- Emits function prototypes and function bodies.
- Emits MIR control flow directly as labels and gotos:
  - `bbN:` labels
  - `goto` for edges
  - `if (...) goto ...; else goto ...;` for MIR branches
  - lowered switch targets as `if` chains + fallback goto
- Emits `let` instructions as mutable local assignments.

## Why this matters
- Establishes the MIR -> backend handoff for real target code generation.
- Provides a concrete backend artifact that can be inspected and compiled externally.
- Keeps backend progress incremental while preserving existing MIR diagnostics/introspection workflows.

## Tradeoffs
- Type mapping is intentionally minimal (`Int`/`Int64` -> `int64_t`, `Bool` -> `bool`, fallback -> `int64_t`).
- The emitter is currently syntactic and not yet ABI/runtime complete.
- Pattern/switch lowering is intentionally simple in this slice and will need deeper semantic typing in later M6 work.

## Next step
- Add compile/link orchestration via `clang` so emitted C can be built into a runnable binary from the CLI.
