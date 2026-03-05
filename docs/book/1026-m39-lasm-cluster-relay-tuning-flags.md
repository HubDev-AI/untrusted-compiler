# 1026 M39 Slice: LASM Cluster Relay Tuning Flags

This slice adds runtime controls for the LASM front-proxy relay layer so scaling behavior can be tuned without code changes.

## What changed

1. Added `sec4 run` flags:
   - `--cluster-relay-workers <N>`
   - `--cluster-relay-queue <N>`
2. Integrated both flags into `LasmClusterConfig` and proxy relay sizing.
3. Added deterministic guardrails:
   - both flags are LASM-only,
   - both flags require cluster mode (`--instances > 1`),
   - fixed reuse-port cluster mode rejects them because the front proxy is bypassed there.

## Why

Default relay sizing works for baseline use, but real load profiles vary. Exposing worker/queue knobs lets operators tune front-layer behavior directly for different latency and memory tradeoffs.

## Validation

Targeted command checks:

- `run_command_rejects_cluster_relay_workers_with_c_backend`
- `run_command_rejects_cluster_relay_queue_without_cluster_mode`
- `run_command_lasm_cluster_mode_serves_request`

These checks confirm deterministic guard behavior and that the cluster serving path remains healthy.
