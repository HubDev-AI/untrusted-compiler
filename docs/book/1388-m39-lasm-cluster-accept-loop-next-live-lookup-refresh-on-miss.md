# M39: LASM Cluster Accept-Loop Next-Live Lookup Refresh-On-Miss

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added `lookup_lasm_cluster_next_live_sender_index_with_refresh_on_miss(...)`,
  - degraded-mode dispatch-cursor realignment now uses this helper,
  - degraded-mode next-dispatch index resolution now uses the same helper.
- The helper behavior is:
  - run cached next-live lookup once,
  - on miss, refresh the lookup table in place,
  - retry lookup once before falling back to existing unavailable/realign behavior.

## Why

During topology churn, next-live lookup cache entries can become temporarily stale. The old flow immediately fell back to slower dead-index paths on a miss. A one-time refresh-and-retry keeps the fast lookup path hot without changing deterministic failure behavior.

## Result

- Degraded dispatch path self-heals stale lookup cache state on first miss.
- Fewer repeated lookup-miss dead-index fallbacks under relay liveness churn.
- Existing deterministic unavailable/saturation behavior remains unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
