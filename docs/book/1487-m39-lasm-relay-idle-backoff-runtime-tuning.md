# M39 Slice: LASM Relay Idle-Backoff Runtime Tuning

Date: 2026-02-23
Milestone: M39-S2B (LASM async backend bootstrap, scale hardening lane)

## What changed

- Added runtime resolver for relay idle-backoff cap:
  - `SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX`
  - default: `1`
  - clamp range: `0..32`
- Relay pump now tracks per-connection idle backoff budget:
  - after an actual idle pump pass, the connection is deferred for `idleBackoffMax` pump ticks,
  - any real IO progress resets backoff immediately.
- Wired resolved value through cluster startup into relay worker loops.
- Exposed resolved value in cluster status JSON as:
  - `relayIdleBackoffMax`

## Why

- Proxy relay hot-path spent repeated nonblocking `WouldBlock` cycles on idle keep-alive connections.
- A bounded idle backoff reduces no-op pump churn while preserving deterministic connection behavior.
- Operators need a runtime knob + status visibility to tune throughput/latency tradeoffs during saturation runs.

## Behavioral contract

- Relay semantics remain deterministic:
  - read/write/error/close behavior is unchanged,
  - only idle scheduling cadence is tuned.
- `SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX=0` disables deferral and restores immediate per-tick probing.
- Status JSON now includes `relayIdleBackoffMax` so effective tuning is auditable.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_`
