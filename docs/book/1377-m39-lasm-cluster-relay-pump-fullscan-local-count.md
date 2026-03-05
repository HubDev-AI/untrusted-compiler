# M39: LASM Cluster Relay Pump Full-Scan Local Count

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - full-scan relay pump scheduler now initializes local mutable `relay_count`,
  - loop condition now uses `index < relay_count` instead of reading `relay_connections.len()` each iteration,
  - removal path decrements `relay_count` while non-removal path advances index.

## Why

After pump-step helper unification, full-scan loop still re-read vector length on every iteration. Local relay-count tracking keeps traversal semantics unchanged while removing repeated length reads from the hot path.

## Result

- Full-scan relay pump uses one local relay-count budget.
- Deterministic index/removal traversal behavior is preserved.
- No change to warning, release, or progress semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
