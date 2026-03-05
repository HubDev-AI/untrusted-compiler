# M39: LASM Cluster Reuse-Port Fast-Path Override

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/main.rs`:
  - cluster fixed reuse-port mode gate now treats explicit `--reuse-port` as an override in cluster mode,
  - `fixed_cluster_reuse_port_mode` now resolves as:
    - `cluster_mode && (max_instances == instances || reuse_port)`.
- Added command test in `$REPO_ROOT/compiler/sec4-cli/tests/commands.rs`:
  - `run_command_rejects_cluster_status_json_when_reuse_port_forces_fixed_cluster_mode`.

## Why

The proxy-relay cluster path is useful for autoscale orchestration, but it has measurable overhead versus fixed reuse-port mode in high-throughput scenarios. Operators needed an explicit way to select the fast path without rewriting instance/autoscale flag sets.

## Result

- Cluster mode keeps existing default behavior when `--reuse-port` is not set.
- In cluster mode, `--reuse-port` now explicitly opts into fixed reuse-port path.
- Proxy-only options keep deterministic guard behavior under this forced fast path (`--cluster-status-json` and other relay-proxy knobs remain rejected in fixed reuse-port mode).

## Validation

- `cargo test -p sec4 --test commands run_command_rejects_cluster_status_json_when_reuse_port_forces_fixed_cluster_mode`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
