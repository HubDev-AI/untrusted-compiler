# 1255 M39 Slice: LASM Cluster Short-Circuit Status Telemetry

This slice makes the relay saturation short-circuit behavior visible in cluster status snapshots.

## What changed

1. Added dedicated short-circuit counter tracking in the LASM cluster accept path:
   - `run_lasm_cluster_accept_loop(...)` now tracks and flushes a local short-circuit total whenever it skips fallback sender scans after a batch-level saturation signal.
2. Added status snapshot/payload fields:
   - `relayDispatchSaturationShortCircuitTotal`
   - `relayDispatchSaturationShortCircuitPerSec`
3. Kept existing dispatch telemetry:
   - Existing fallback metrics (`relayDispatchFallbackTotal`, `relayDispatchFallbackPerSec`) remain unchanged.
   - New telemetry is additive and deterministic.
4. Extended status-json integration coverage:
   - `run_command_lasm_cluster_status_json_skips_unchanged_snapshots` now asserts the new status fields are present.

## Why

The previous optimization reduced fallback-scan churn under saturation, but operators could not observe when and how often that path was used. These fields make short-circuit behavior measurable in the same status artifact used for autoscale/proxy tuning.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/tests/commands.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots -- --exact`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_honors_max_pending_override_before_overflow -- --exact`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 10s --threads 4 --connections 128 --target-requests 200000 --out results/summaries/sec4-lasm-cluster-capacity-probe-short-circuit-telemetry.json`
5. `jq '{pass, observed: {requestsPerSec: .observed.requestsPerSec, p99: .observed.p99, peakRssKb: .observed.peakRssKb}}' benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-short-circuit-telemetry.json`

Short local sample from step 5:
- `pass=true`
- `requestsPerSec ~ 79288`
- `p99=2.77ms`
- `peakRssKb=19776`
