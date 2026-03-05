# M39: LASM Cluster Unavailable Response Helper Split

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_dispatch.rs`:
  - replaced reason-enum + shared dispatch helper with dedicated response writers:
    - `write_lasm_cluster_no_healthy_workers_response(...)`
    - `write_lasm_cluster_worker_unavailable_response(...)`
  - kept static response buffers and error text unchanged.
- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - relay worker now calls dedicated unavailable-response helpers directly.

## Why

Unavailable-response paths were still routing through a reason-enum dispatch layer. Dedicated entry points simplify hot failure-path plumbing while keeping deterministic response behavior.

## Result

- Reduced unavailable-response dispatch indirection.
- No behavior change in response payloads or failure semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
