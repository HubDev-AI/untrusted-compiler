# M39: LASM Cluster Accept-Loop Flush-Helper Non-Zero Gates

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - batch-tail helper flush calls are now gated on non-zero local counters:
    - active connection increments,
    - dispatch fallback totals,
    - dispatch short-circuit totals.
  - final loop tail helper flush calls are now gated similarly for:
    - saturation counters,
    - fallback totals,
    - short-circuit totals.

## Why

Accept-loop hot path can execute many iterations where local counters are unchanged. Gating helper calls on non-zero counters removes no-op helper-call overhead in steady-state operation.

## Result

- Reduced no-op helper-call overhead in accept-loop cycle tails.
- No change to counter-flush semantics when local deltas are present.
- Existing saturation and dispatch telemetry contracts remain deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
