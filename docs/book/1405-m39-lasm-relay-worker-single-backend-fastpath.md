# M39: LASM Relay Worker Single-Backend Fast Path

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated relay-worker backend selection path in `compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - bypass generic selection lookup/recompute machinery when backend count is `0` or `1`
  - resolve one-backend routing directly from unhealthy state (`unhealthy_port_count`)
  - keep multi-backend selection path unchanged

## Why

The generic selection pipeline is designed for multi-backend routing, but small deployments with one backend still paid part of that branch/recompute overhead. A direct fast path reduces overhead for the common single-backend case without changing behavior.

## Result

- `worker_port_count == 1` now routes directly to backend `0` only when healthy; otherwise returns deterministic unavailable behavior.
- Multi-backend selection/reservation logic remains unchanged.
- Hot-path branch/lookup work is reduced in small-cluster configurations.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
