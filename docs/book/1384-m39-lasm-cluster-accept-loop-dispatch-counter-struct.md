# M39: LASM Cluster Accept-Loop Dispatch Counter Struct

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`:
  - added `LasmClusterAcceptDispatchCounters` to group saturation/fallback/short-circuit local counters,
  - updated unavailable-stream and dispatch-counter flush helpers to consume this counter struct,
  - rewired accept-loop dispatch/error call sites to use shared counter state instead of passing four separate local counter references each time.

## Why

Accept-loop hot path carried repeated multi-counter argument threading across many branches. Grouping those counters in one local struct reduces call-site complexity and keeps dispatch counter state transitions centralized.

## Result

- Accept-loop helper signatures are shorter and easier to maintain.
- Counter updates remain deterministic and aligned with existing flush behavior.
- No change to dispatch error envelopes or accept-loop control flow.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
