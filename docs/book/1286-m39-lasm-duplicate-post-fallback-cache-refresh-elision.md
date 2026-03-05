# M39: LASM Duplicate Post-Fallback Cache Refresh Elision

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated fallback dispatch section in `run_lasm_cluster_accept_loop(...)` (`/compiler/sec4-cli/src/main.rs`).
- Added `live_count_after_primary_dispatch` checkpoint.
- Post-fallback next-live cache refresh now runs only when:
  - cache is eligible (`relay_has_next_live_sender_lookup` and `relay_live_sender_count > 2`), and
  - fallback changed live-count beyond the already-handled primary-disconnect state (`relay_live_sender_count != live_count_after_primary_dispatch`).

## Why

After immediate cache refresh on primary disconnect, post-fallback logic could refresh the same cache again even when fallback did not change liveness. That duplicates O(n) refresh work.

## Result

- Removed redundant second cache refresh for primary-disconnect-only fallback flows.
- Preserved cache refresh for additional disconnects that occur inside fallback dispatch.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
