# M39 - CLI LASM Smoke Command

## What Was Added

Added a CLI entrypoint that executes the LASM runtime baseline directly.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

Command:

- `sec4 lasm-smoke --path <project> [--method GET] [--route /health] [--requests 1] [--max-steps 128]`

## Behavior

1. Validates and analyzes project entry (including multi-file modules).
2. Lowers program to MIR and LASM.
3. Uses LASM entry metadata to register/run an in-memory HTTP request on `LasmHttpRuntime`.
4. Emits deterministic execution summary (`entry`, `origin`, `requests`, `ok`, `errors`, `steps`, `nowMs`, `body`).
5. Supports batched in-memory execution via `--requests <N>` and emits deterministic `ok`/`errors` counters.

Failure guards:

1. rejects `--requests 0`,
2. rejects `--max-steps 0`,
3. rejects empty method/route values,
4. fails if no LASM entrypoint is available,
5. fails if runtime does not produce expected response count.

## Why

This exposes the LASM bootstrap lane through a user-visible CLI flow so runtime evolution is testable without C backend/socket dependencies.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_runs_in_memory_runtime_with_compiled_entrypoint`
2. `cargo test -p sec4 --test commands lasm_smoke_command_rejects_zero_step_budget`
3. `cargo test -p sec4 --test commands lasm_smoke_command_rejects_zero_request_count`
4. `cargo test -p sec4 --test commands lasm_smoke_command_fails_when_step_budget_is_too_low_for_batch`
5. `cargo test -p sec4 --test commands`
