# M39 - c-bin Canonical Runtime Source Selection

## What Was Changed

`sec4 build --emit c-bin` now prefers canonical runtime sources from the repository runtime directory:

1. `runtime/c/sec4_runtime.c`
2. `runtime/c/sec4_runtime.h`

Implementation file:

- `compiler/sec4-cli/src/main.rs`

Test update:

- `compiler/sec4-cli/tests/json_output.rs`

## Behavior

Compilation path in `compile_c_binary` now works as:

1. use canonical runtime files if they exist,
2. fallback to writing runtime assets into `build/` only when canonical files are unavailable.

This keeps local project build directories cleaner while preserving deterministic fallback behavior.

## Why

This aligns runtime compilation with the single source of truth (`runtime/c/*`) and reduces confusion about why runtime files appear in each project build folder.

## Validation

1. `cargo test -p sec4 --test json_output build_emit_c_bin_compiles_binary_when_clang_available`
2. `cargo test -p sec4 --test commands run_command_oneshot_serves_request_and_exits`
3. `cargo test -p sec4 --test commands`
