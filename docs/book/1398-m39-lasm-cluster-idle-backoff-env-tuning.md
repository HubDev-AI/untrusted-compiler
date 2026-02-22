# M39: LASM Cluster Idle Backoff Env Tuning

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added env-resolved idle backoff controls in
  `compiler/sec4-cli/src/lasm_cluster_runtime_config.rs`:
  - `resolve_lasm_cluster_idle_spin_threshold()`
  - `resolve_lasm_cluster_idle_sleep_micros()`
- New runtime env knobs:
  - `SEC4_RT_LASM_CLUSTER_IDLE_SPIN_THRESHOLD`
    - default: `32`
    - clamp: `1..4096`
  - `SEC4_RT_LASM_CLUSTER_IDLE_SLEEP_MICROS`
    - default: `250`
    - clamp: `1..50000`
- Wired both LASM hot loops to these runtime-config resolvers:
  - `compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`
  - `compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`

## Why

Accept/relay idle backoff values were hardcoded. That forced source edits for every tuning attempt when balancing latency/CPU under different machines and load profiles. This slice makes the backoff policy tunable at runtime while preserving deterministic defaults.

## Result

- Existing behavior is unchanged by default.
- Operators can tune idle spin/sleep behavior without rebuilding `sec4`:
  - lower sleep/higher spin for lower-latency wakeups,
  - higher sleep/lower spin for reduced idle CPU.
- Throughput tuning loops can now run as env-only experiments.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_cluster_runtime_config.rs compiler/sec4-cli/src/lasm_cluster_accept_loop.rs compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`
- `cargo test -p sec4 --test commands run_command_oneshot_serves_request_and_exits`
