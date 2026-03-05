# M39: LASM Cluster Accept-Loop Live-Start Fallback On Next-Dispatch Lookup Miss

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - in degraded-liveness next-dispatch cursor resolution (`relay_live_sender_count > 2` path),
  - after next-live lookup miss (including refresh-on-miss helper path),
  - changed fallback cursor from `next_dispatch_wrapped` to `stream_dispatch_start`.

## Why

`next_dispatch_wrapped` can be a known dead slot in degraded mode when lookup cache is stale. Keeping that dead slot as the next primary cursor causes avoidable immediate primary-dispatch failures on subsequent requests. `stream_dispatch_start` is the safest known-live cursor at selection time.

## Result

- Fewer avoidable dead-slot primary dispatch attempts in degraded liveness windows.
- Faster convergence back to healthy primary dispatch behavior after lookup misses.
- Existing deterministic saturation/unavailable fallback behavior remains unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
