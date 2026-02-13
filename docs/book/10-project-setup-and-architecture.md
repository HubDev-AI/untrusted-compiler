# 10 Project Setup and Architecture (M0)

## Repository layout

- `compiler/`
- `runtime/`
- `stdlib/`
- `examples/`
- `docs/`

## Compiler workspace

The compiler is a Rust workspace with two crates:
- `compiler/sec4-core`: shared compiler foundations (manifest + diagnostics for M0).
- `compiler/sec4-cli`: command-line entrypoint and M0 command routing.

## Why this split now

- Keeps CLI concerns separate from compiler logic.
- Allows independent tests for core logic without invoking full binary flows.
- Scales cleanly into parser/typechecker/backend phases.

## Current boundaries (M0)

- Real compilation pipeline is not implemented yet.
- `build`/`check` validate project structure and emit diagnostics.
- Runtime/stdlib are placeholders to lock directory contracts early.
