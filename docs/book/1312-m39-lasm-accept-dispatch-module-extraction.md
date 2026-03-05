# M39: LASM Accept Dispatch Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `$REPO_ROOT/compiler/sec4-cli/src/lasm_cluster_accept_dispatch.rs`.
- Moved accept-dispatch helpers out of `main.rs`:
  - unavailable response enum/constants/writer:
    - `LasmClusterUnavailableReason`
    - `write_lasm_cluster_unavailable_response(...)`
  - counter flush helpers:
    - `flush_lasm_cluster_saturation_counters(...)`
    - `flush_lasm_cluster_dispatch_fallback_total(...)`
    - `flush_lasm_cluster_dispatch_short_circuit_total(...)`
    - `flush_lasm_cluster_active_connection_increments(...)`
    - `flush_lasm_cluster_active_connection_decrements(...)`
  - dispatch error mapper:
    - `handle_lasm_cluster_accept_dispatch_error(...)`
  - saturation flush batch constant:
    - `LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH`
- `main.rs` now imports these helpers from module boundaries.

## Why

Accept-dispatch error/flush logic was still a dense hot-path block in `main.rs`. Extracting this block isolates overload/unavailable response behavior and flush mechanics into a dedicated module while keeping accept-loop orchestration in the entry file.

## Result

- `main.rs` is reduced to orchestration and call-site flow.
- Accept-dispatch semantics remain unchanged (same overload envelopes and counter flush behavior).

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
