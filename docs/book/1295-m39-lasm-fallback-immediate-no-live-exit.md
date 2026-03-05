# M39: LASM Fallback Immediate No-Live Exit

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` in `/compiler/sec4-cli/src/main.rs`.
- In the degraded general scan loop (`scan_live_target > 2` path), added an immediate terminal check after disconnect handling:
  - when `relay_live_sender_count` reaches `0`, fallback now returns `LasmClusterRelayDispatchError::Unavailable(...)` immediately.

## Why

When disconnect handling drops live relay senders to zero, fallback cannot recover a dispatch target in that request path. Continuing through remaining loop-control checks and scan-index advancement work adds unnecessary hot-path branching.

## Result

- Immediate deterministic exit on terminal no-live transitions in degraded fallback scan.
- Preserved saturated/unavailable envelope behavior and disconnect-driven liveness tracking.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
