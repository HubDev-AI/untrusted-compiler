# M39: LASM Degraded Fallback Cached-Hint Fast Paths

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated degraded fallback selection in `/compiler/sec4-cli/src/main.rs` accept loop.
- Added `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)` helper.
- Fallback selection now:
  - for `live_count == 2`, refreshes missing dual-live hints once and prefers dual-live helper path,
  - for `live_count == 1`, refreshes missing single-live hint once and prefers single-live helper path,
  - uses generic `fallback_multi` path only when dedicated hint-based paths are unavailable.

## Why

Even in degraded state, generic fallback scans were still used on some single/dual-live paths when hints were missing, adding avoidable per-request scanning overhead.

## Result

- Dedicated single/dual-live fallback helpers are used more consistently.
- Degraded fallback now does at most one hint refresh before helper dispatch.
- Deterministic error mapping and relay liveness transitions remain unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
