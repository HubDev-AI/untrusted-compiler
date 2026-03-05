# M39: LASM Autoscale Recovery Port Buffer Reuse

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Updated min-worker recovery port reservation API to fill a caller-provided buffer:
  - `reserve_lasm_cluster_min_worker_ports(...)` now writes directly into the autoscale loop’s reusable spawn-port vector
  - autoscale loop no longer allocates a temporary recovery-port vector each iteration

## Why

After adding autoscale loop buffer reuse, recovery reservation still created a temporary vector and then copied into the reusable spawn buffer. This left avoidable allocation/copy churn in the recovery path.

## Result

- Recovery port reservation now uses the same reused spawn-buffer path.
- Per-iteration recovery reservation avoids extra temporary vector allocation.
- Port reservation semantics remain deterministic.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
