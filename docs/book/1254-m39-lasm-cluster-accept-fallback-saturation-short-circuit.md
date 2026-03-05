# 1254 M39 Slice: LASM Cluster Accept Fallback Saturation Short-Circuit

This slice reduces redundant relay-dispatch work in the LASM cluster proxy accept path under heavy queue saturation.

## What changed

1. Relay accept-loop fast path:
   - File: `compiler/sec4-cli/src/main.rs`
   - Function: `run_lasm_cluster_accept_loop(...)`
   - In multi-sender proxy mode, the accept loop now tracks a per-batch saturation hint:
     - when a fallback scan returns `Saturated`, the batch is marked as fully saturated,
     - subsequent `TrySendError::Full` events in that same batch skip repeating the full fallback scan and directly use deterministic saturation handling.
2. Immediate recovery behavior:
   - Any successful primary/fallback dispatch in the batch clears the hint so normal fallback scanning resumes as soon as capacity is available again.
3. Scope:
   - No CLI surface changes.
   - No response contract changes.
   - Applies only to the multi-sender proxy dispatch branch.

## Why

When queues are already saturated, repeatedly scanning all relay sender shards per incoming connection adds avoidable CPU overhead in the accept hot path. This slice keeps the same saturation behavior while reducing repeated scan work inside one accept batch.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_honors_max_pending_override_before_overflow -- --exact`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_max_pending_from_policy_before_overflow -- --exact`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 10s --threads 4 --connections 128 --target-requests 200000 --out results/summaries/sec4-lasm-cluster-capacity-probe-hotpath-short.json`
5. `jq '{pass, observed: {requestsPerSec: .observed.requestsPerSec, p99: .observed.p99, peakRssKb: .observed.peakRssKb}}' benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-hotpath-short.json`

Short local sample from step 5:
- `pass=true`
- `requestsPerSec ~ 75772`
- `p99=7.01ms`
- `peakRssKb=19568`
