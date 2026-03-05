# M39 Slice: LASM Relay Pump Scan Multiplier Runtime Tuning

Date: 2026-02-23
Milestone: M39-S2B (LASM async backend bootstrap, scale hardening lane)

## What changed

- Added runtime resolver for relay batch-mode scan multiplier:
  - `SEC4_RT_LASM_CLUSTER_RELAY_PUMP_SCAN_MULTIPLIER`
  - default: `4`
  - clamp range: `1..16`
- Relay worker loop now uses this multiplier when calculating batch-mode `scan_budget` for relay pumping.
- Wired the resolved value through cluster startup into relay workers.
- Exposed effective value in cluster status JSON as:
  - `relayPumpScanMultiplier`

## Why

- Backoff-skip-aware scheduling benefits from tuning how aggressively each relay worker scans past deferred-idle connections before ending a tick.
- A runtime knob allows operators to tune fairness/CPU trade-offs per host profile without code changes.

## Behavioral contract

- `pump_budget` remains bounded by existing relay pump batch limits.
- `scan_budget` remains bounded and deterministic; multiplier only affects batch-mode scan reach.
- `relayPumpScanMultiplier` in status JSON reflects the active runtime value used by workers.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_`

