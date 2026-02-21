# M39: LASM No-Live Telemetry Atomic Elision

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated LASM cluster accept loop in `/compiler/sec4-cli/src/main.rs`.
- Removed repeated `relay_live_sender_count_observed.fetch_min(0, ...)` writes from the no-live branch (`relay_live_sender_count == 0`).

## Why

In fully degraded no-live states, the accept loop could perform redundant per-request atomic updates even though live-count transitions are already captured when sender liveness changes.

## Result

- Reduced no-live degraded-path atomic overhead.
- Preserved telemetry correctness: transition-to-zero is still recorded at the point where live sender count changes.
- No change to deterministic unavailable response behavior.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
