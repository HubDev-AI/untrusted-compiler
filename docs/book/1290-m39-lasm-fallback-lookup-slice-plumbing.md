# M39: LASM Fallback Lookup Slice Plumbing

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/compiler/sec4-cli/src/main.rs`:
  - `resolve_lasm_cluster_next_live_sender_index(...)` now accepts direct lookup slice input (`&[usize]`) and treats empty slices as no-cache fallback,
  - `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` now accepts direct lookup slice input (`&[usize]`) instead of option-wrapped lookup references.
- Accept-loop fallback call path now passes:
  - real lookup slice for eligible cached degraded states (`live_count > 2`),
  - empty lookup slice otherwise.

## Why

Option-wrapped lookup plumbing in fallback dispatch added extra branch wrapping/unwrapping in a hot path where empty-slice semantics already represent “no cache”.

## Result

- Simpler fallback resolver/call-site shape.
- Reduced option wrapping overhead in degraded fallback dispatch paths.
- Preserved deterministic fallback behavior through unchanged empty-slice scan fallback semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
