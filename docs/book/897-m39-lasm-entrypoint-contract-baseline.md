# M39 - LASM Entrypoint Contract Baseline

## What Was Added

Extended LASM emission with deterministic runtime entrypoint metadata.

Files:

1. `compiler/sec4-core/src/lasm_backend.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Contract

LASM output now carries an explicit entrypoint contract instead of relying on implicit function ordering.

Selection rule:

1. prefer function named `main`,
2. fallback to first function if `main` is absent,
3. no entry metadata when no functions exist.

Text emit now includes:

- `.entry <name> params=<count> ret=<type>`

JSON emit now includes:

- `entry: { name, param_count, return_type }`

## Why

The async LASM runtime lane needs a stable handoff contract from compiler output to runtime launcher. Explicit entry metadata removes launch ambiguity and keeps runtime bootstrap deterministic.

## Validation

1. `cargo test -p sec4-core lasm_backend::tests::`
2. `cargo test -p sec4 --test commands build_emit_lasm_outputs_lasm_text`
3. `cargo test -p sec4 --test commands build_emit_lasm_json_outputs_machine_readable_lasm`
