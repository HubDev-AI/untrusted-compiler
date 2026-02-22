# M39: LASM Cluster Accept-Loop Stream-Local Live-Hints Fast Path

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - introduced per-stream local hint slots (`stream_single_live_index`, `stream_dual_live_indices`) in the multi-sender accept path,
  - set those hints once during degraded-liveness cursor-prep (`live_count == 1` / `live_count == 2`),
  - reused the pre-resolved hints in `next_dispatch_index` resolution instead of re-checking shared option state in the two-live branch.

## Why

In partial-liveness mode, the accept loop already resolves single/dual live sender hints before dispatching each stream. The next-dispatch index path still re-ran option-based branch checks against shared state. Reusing stream-local hints removes that repeated branch path from a hot section without changing fallback/unavailable behavior.

## Result

- Degraded-liveness dispatch index selection now reuses one stream-local hint decision.
- Two-live next-index selection no longer depends on fallback option checks in the hot branch.
- Existing deterministic saturation/unavailable/error envelope behavior is unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
