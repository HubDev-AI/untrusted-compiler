# 80 M6 Slice: `run` Command via C Backend

This chapter documents the next M6 vertical slice: wiring `ailang run` to the C backend pipeline.

## Scope delivered
- `ailang run --path <project>` now:
  1. builds through `--emit c-bin` pipeline,
  2. executes the compiled binary from `build/<package-name>`.
- Added integration coverage for `run` execution (clang-gated).

## What changed
- `cmd_run` is no longer a placeholder.
- `cmd_run` now validates project manifest, invokes C binary build flow, and executes produced binary.
- Non-zero binary exit status is propagated as CLI failure.

## Why this matters
- Establishes first practical end-to-end execution path for AILang projects.
- Confirms MIR + C emission + clang pipeline can power the user-facing `run` flow.
- Makes runtime iteration possible before full runtime ABI/intrinsic coverage lands.

## Tests added
- CLI integration test:
  - `run_command_executes_compiled_binary_when_clang_available`

## Tradeoffs
- `run` currently depends on host `clang` availability.
- Runtime behavior still reflects the current minimal language/backend subset.
- Output is build-oriented; future UX passes can streamline build/run messaging.

## Next step
- Add explicit runtime ABI contracts and intrinsic linking so effectful/stdlib-heavy programs can execute through `run`.
