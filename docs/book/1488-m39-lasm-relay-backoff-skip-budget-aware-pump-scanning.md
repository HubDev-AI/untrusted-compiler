# M39 Slice: LASM Relay Backoff-Skip Budget-Aware Pump Scanning

Date: 2026-02-23
Milestone: M39-S2B (LASM async backend bootstrap, scale hardening lane)

## What changed

- Added explicit relay pump step for deferred idle-backoff scans:
  - `LasmClusterRelayPumpStep::BackoffDeferred`
- Relay `pump_once()` now emits `BackoffDeferred` when idle-backoff countdown is active.
- Updated relay worker pump loop scheduling:
  - deferred-backoff scans no longer consume the main `pump_budget`,
  - loop now enforces a separate bounded `scan_budget` to keep each tick finite.
- Batch-mode scan budget is now bounded by `min(relay_count, relay_pump_batch_max * 4)` so workers can skip over deferred-idle connections and still service active relays in the same tick.

## Why

- With many keep-alive idle relays, deferred-idle entries could consume the whole per-tick pump budget before active relays were reached.
- This change preserves finite tick work while improving active-connection service fairness under mixed idle/active traffic.

## Behavioral contract

- No change to relay wire semantics (`read/write/error/close`).
- Only scheduler accounting changed:
  - `Idle` (actual non-deferred probe) still consumes pump budget,
  - `BackoffDeferred` does not consume pump budget but still consumes scan budget.
- Tick work remains bounded in both full-scan and batch modes.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_`

