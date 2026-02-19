# 1060 M39 Slice: LASM Cluster Relay Read Drain Loop

This slice optimizes the relay pump read path by draining readable socket data in one cycle instead of taking a single read per direction per cycle.

## What changed

1. Updated `LasmClusterRelayPump::pump_once()` in `compiler/sec4-cli/src/main.rs`.
2. For both relay directions:
   - `client -> upstream` buffer fill phase,
   - `upstream -> client` buffer fill phase,
   the code now loops reads until `WouldBlock`, EOF, or local buffer is full.
3. Existing contracts remain unchanged:
   - bounded per-direction buffer usage,
   - deterministic write-side half-close behavior,
   - existing error/availability response envelopes outside the relay pump.

## Why

Single-read-per-cycle behavior left readable socket bursts to be consumed across multiple scheduling cycles. Draining readable bytes per cycle reduces loop churn and improves data movement efficiency for active relay pairs.

## Validation

1. Focused command test:
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. Short cluster probe:
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 400000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-latest-read-drain-short.json`

Comparison against current post-1059 baseline:

- baseline (`port-cooldown-fastpath-short`):
  - `requestsPerSec`: `72,490.70`
  - `requests`: `1,457,249`
  - `peakRssKb`: `21,888`
  - `p99`: `9.54ms`
- this slice (`read-drain-short`):
  - `requestsPerSec`: `72,761.61`
  - `requests`: `1,462,766`
  - `peakRssKb`: `22,192`
  - `p99`: `9.60ms`
