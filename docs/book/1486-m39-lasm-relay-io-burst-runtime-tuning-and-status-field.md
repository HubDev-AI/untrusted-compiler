# M39 Slice: LASM Relay IO Burst Runtime Tuning and Status Visibility

Date: 2026-02-23
Milestone: M39-S2B (LASM async backend bootstrap, scale hardening lane)

## What changed

- Added runtime resolver for relay per-tick IO burst cap:
  - `SEC4_RT_LASM_CLUSTER_RELAY_IO_BURST_MAX`
  - default: `4`
  - clamp range: `1..64`
- Wired resolved value through cluster runtime startup into relay worker loops.
- Relay pump now consumes the resolved per-connection burst cap instead of a fixed compile-time value.
- Exposed resolved value in cluster status JSON as:
  - `relayIoBurstMax`

## Why

- The previous fairness cap was hardcoded.
- Operators need to tune fairness vs. per-connection throughput without code changes during load runs.
- Status JSON should show the effective runtime value so benchmark evidence is reproducible and auditable.

## Behavioral contract

- Existing relay semantics are unchanged:
  - nonblocking proxy relay behavior remains intact,
  - completion and error paths are unchanged,
  - only per-tick IO burst budget is now runtime-configurable.
- Status JSON now includes one additional deterministic tuning field: `relayIoBurstMax`.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
