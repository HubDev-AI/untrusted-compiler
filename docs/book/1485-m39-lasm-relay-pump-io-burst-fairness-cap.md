# M39 Slice: LASM Relay Pump IO Burst Fairness Cap

Date: 2026-02-23
Milestone: M39-S2B (LASM async backend bootstrap, scale hardening lane)

## What changed

- Added a bounded per-direction IO burst cap in `compiler/sec4-cli/src/lasm_cluster_relay_pump.rs`:
  - `LASM_CLUSTER_RELAY_IO_BURST_MAX = 4`
  - Applies to all four relay pump loops in one `pump_once()` tick:
    - client read -> relay buffer
    - relay buffer write -> upstream
    - upstream read -> relay buffer
    - relay buffer write -> client

## Why

- The relay pump previously kept reading/writing a single connection until `WouldBlock` or buffer exhaustion.
- Under uneven traffic, one busy connection could consume too much worker time in a single tick and delay progress on other active relay connections.
- Bounding per-direction loop bursts keeps each connection's tick work finite and improves worker fairness on hot paths.

## Behavioral contract

- No protocol semantics changed:
  - nonblocking relay behavior is preserved,
  - half-close handling and completion detection are unchanged,
  - backpressure/error behavior remains deterministic.
- The cap only limits per-tick work; pending bytes continue on later pump ticks.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`

## Tradeoffs

- Lower per-tick work on one connection can increase tick count for very large payload relays.
- In return, cluster workers avoid single-connection monopolization and maintain better progress fairness across many concurrent relay sessions.
