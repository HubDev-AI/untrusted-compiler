# M39: LASM Relay Fallback Start Cursor Precompute

Date: 2026-02-22  
Milestone: M39 (proxy/runtime hot-path tuning)

## What Changed

- Added relay-worker precompute for non-identity fallback start cursors:
  - builds a per-backend fallback-start cursor map when selection lookup state is rebuilt
  - stores that map for direct lookup during fallback scans
- Updated degraded-path fallback flow to use the precomputed start cursor rather than recalculating via binary search on each connect failure.

## Why

Fallback scans in degraded clusters previously recomputed fallback start cursors with `binary_search` inside per-failure loops. This creates avoidable repeated lookup work under backend-connect failure pressure.

## Result

- Fallback ordering behavior is unchanged.
- Per-failure degraded-path fallback scans avoid repeated binary-search cursor resolution.
- Selection-state recompute cost absorbs cursor-map construction once per lookup rebuild.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
