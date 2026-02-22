# M39: LASM Cluster Accept/Fallback Direct Wrap-Step Elision

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - replaced `lasm_cluster_next_index_wrapped(...)` usage for `next_dispatch_wrapped` with direct increment+wrap arithmetic.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`:
  - replaced dual-live second-attempt `lasm_cluster_next_index_wrapped(...)` usage with direct increment+wrap arithmetic.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`:
  - removed now-unused `lasm_cluster_next_index_wrapped` helper.

## Why

These call sites are in request-dispatch paths where sender count is already known and bounded. Direct wrap-step arithmetic removes helper indirection and keeps branch behavior explicit in the hot path.

## Result

- Fewer helper calls in accept/fallback dispatch routing.
- No runtime behavior change in cursor progression.
- Removed dead symbol from relay-topology module.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
