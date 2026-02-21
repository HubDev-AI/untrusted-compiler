# M39: LASM Next-Live Lookup Sparse Activation

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `run_lasm_cluster_accept_loop(...)` in `/compiler/sec4-cli/src/main.rs`.
- `relay_next_live_sender_lookup` is now allocated only when relay sender count is greater than two.
- Lookup refresh on liveness-change now runs only when:
  - live sender count changed, and
  - new live sender count is still greater than two.

## Why

Dedicated fast paths already handle `live_count == 1` and `live_count == 2`. Keeping next-live lookup cache allocation and refresh active in those states adds unnecessary maintenance work with no dispatch benefit.

## Result

- Reduced degraded-mode bookkeeping for two-shard and single-shard states.
- Preserved deterministic relay dispatch semantics and existing single/dual specialized behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
