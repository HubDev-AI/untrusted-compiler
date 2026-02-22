# M39: LASM Cluster Next-Live Scan Direct Wrap Progression

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`:
  - `lasm_cluster_next_live_sender_index` now advances its scan cursor with direct increment + wrap reset in-loop,
  - removed per-iteration `lasm_cluster_next_index_wrapped(...)` call from this shared scan helper.

## Why

This helper is used in degraded dispatch/topology resolution flows and can be called frequently under unhealthy/saturated conditions. In-loop direct wrap progression trims helper-call overhead while preserving scan behavior.

## Result

- Reduced loop-step overhead in shared next-live sender scan utility.
- No behavior change in next-live resolution outcome.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
