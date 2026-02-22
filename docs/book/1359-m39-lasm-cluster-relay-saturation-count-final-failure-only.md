# M39: LASM Cluster Relay Saturation Count Final-Failure Only

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - moved saturation counter increments from primary connect-failure entry to final worker-unavailable response path.

## Why

Primary connect failures that are recovered by immediate fallback connect should not be counted as saturation outcomes. Counting only final failures keeps saturation telemetry aligned with actual request-level unavailability.

## Result

- Lower false-positive saturation increments under transient single-backend connect faults.
- More accurate autoscale/saturation signals.
- No change to final unavailable behavior or response envelopes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
