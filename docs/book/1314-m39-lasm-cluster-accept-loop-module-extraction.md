# M39: LASM Cluster Accept Loop Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`.
- Moved `run_lasm_cluster_accept_loop(...)` out of `main.rs` into the new module.
- `main.rs` now imports `run_lasm_cluster_accept_loop` from module boundaries.
- Accept-loop module explicitly consumes:
  - accept-dispatch helpers (`lasm_cluster_accept_dispatch`)
  - fallback dispatch helpers (`lasm_cluster_fallback_dispatch`)
  - relay send helpers (`lasm_cluster_relay_send`)
  - relay topology helpers (`lasm_cluster_relay_topology`)

## Why

`run_lasm_cluster_accept_loop(...)` was one of the largest remaining control blocks in `main.rs`. Moving it into a dedicated module continues the multi-file decomposition priority and keeps the CLI entry file focused on high-level runtime wiring.

## Result

- `main.rs` is significantly reduced in LASM cluster proxy control-path size.
- Accept-loop semantics are preserved (same saturation/unavailable behavior, same fallback routing, same counter flush flow).

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
