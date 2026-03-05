# M39: LASM Selection Reservation Min-Chunk Env and Status

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added resolver `resolve_lasm_cluster_selection_reservation_min_chunk(...)` in `/compiler/sec4-cli/src/main.rs`.
- New env override:
  - `SEC4_RT_LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK`
- Reservation chunk sizing now uses:
  - `max(relay_accept_batch_max, resolved_selection_reservation_min_chunk)`
  instead of a compile-time-only minimum.
- Cluster status snapshot/payload now includes:
  - `relaySelectionReservationMinChunk`

## Why

Relay backend selection uses shared atomic reservation chunks (`fetch_add`) for round-robin index allocation. The previous hardcoded lower bound worked but offered no operator tuning path for different load/CPU profiles.

## Result

- Operators can tune reservation chunk sizing without code changes.
- Status JSON exposes the resolved value for reproducible probe runs and report attribution.
- Default behavior remains unchanged when env override is not set.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
