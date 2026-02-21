# M39: LASM Next-Live Lookup Linear Refresh

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `refresh_lasm_cluster_next_live_sender_lookup(...)` in `/compiler/sec4-cli/src/main.rs`.
- Replaced per-slot scan-based refresh (`N` calls to `lasm_cluster_next_live_sender_index`) with a single reverse sweep:
  - anchor on first live shard index,
  - walk slots in reverse cyclic order,
  - carry the most recent live index as the cached next-live target.
- Kept fallback behavior for zero-live state (`lookup[index] = index`) unchanged.

## Why

The previous refresh path was O(n^2) on liveness changes because each slot refresh performed a full live-scan lookup. Liveness-change events should remain lightweight so disconnect recovery overhead does not compound under churn.

## Result

- Lookup refresh is now O(n) per liveness-change update.
- Accept-loop degraded dispatch keeps the same deterministic live-slot lookup semantics with lower refresh overhead.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
