# M39: LASM Cluster Cyclic Healthy Lookup Selection

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_backend_selection.rs`:
  - degraded lookup rebuild now gathers healthy backend indices and fills selection lookup slots with cyclic healthy-index mapping,
  - removed previous next-healthy-from-index mapping that produced selection skew based on unhealthy index gaps.

## Why

Under partial backend failures, gap-weighted mapping can over-route to some healthy workers depending on dead-worker index layout. Cyclic healthy mapping keeps selection distribution deterministic and balanced across remaining healthy backends.

## Result

- More even degraded-mode relay selection across healthy backends.
- Deterministic degraded routing still preserved through stable index order.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
