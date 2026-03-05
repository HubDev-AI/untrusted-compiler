# M39: LASM Cluster Relay Connect Fallback Multi-Alternate Attempts

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`:
  - relay worker connect-failure fallback now scans all remaining healthy backend candidates in cyclic order (starting from failed-backend-next index),
  - each candidate receives one bounded connect attempt before final unavailable response,
  - failed fallback candidates are marked unhealthy with deterministic cooldown/warning handling before continuing scan.

## Why

A single fallback attempt still drops requests when that one alternate candidate is also transiently unavailable, even if other healthy backends exist. Bounded multi-alternate attempts improve resilience without introducing unbounded retry behavior.

## Result

- Better request survival under clustered transient connect failures.
- Fallback attempt order remains deterministic and bounded by worker count.
- Final-failure saturation semantics remain unchanged (saturation increments only on final unavailable outcome).

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
