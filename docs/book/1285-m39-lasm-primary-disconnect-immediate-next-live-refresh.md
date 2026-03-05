# M39: LASM Primary-Disconnect Immediate Next-Live Refresh

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated primary-disconnect handling in `run_lasm_cluster_accept_loop(...)` (`/compiler/sec4-cli/src/main.rs`).
- Primary `try_send` error match now tracks whether the failure was `Disconnected`.
- For eligible cached degraded states (`relay_has_next_live_sender_lookup` and `relay_live_sender_count > 2`), next-live lookup cache is refreshed immediately after primary disconnect and before fallback dispatch.

## Why

Without immediate refresh, fallback dispatch in the same request could run against stale next-live cache entries and hit scan fallback paths until the later post-fallback refresh point.

## Result

- Better cache freshness entering fallback dispatch after primary disconnect.
- Reduced stale-cache fallback scan work in the same request.
- Existing post-fallback liveness refresh behavior remains unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
