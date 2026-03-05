# M39: LASM Cluster Listener Nonblocking Preflight

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `cmd_run_lasm_cluster(...)` to set listener nonblocking immediately after successful bind.
- Removed the late nonblocking failure branch that previously ran after relay/autoscale/status thread startup.

## Why

If nonblocking setup fails, there is no reason to bootstrap worker processes and supporting threads. Moving this setup to preflight keeps failure handling earlier and avoids unnecessary startup/shutdown churn.

## Result

- Failure path is earlier and simpler: nonblocking errors now exit before cluster bootstrap.
- Runtime behavior is unchanged for successful startup and steady-state request handling.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
