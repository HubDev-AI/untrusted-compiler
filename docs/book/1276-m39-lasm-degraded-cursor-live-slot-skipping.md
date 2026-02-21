# M39: LASM Degraded Cursor Live-Slot Skipping

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated degraded-mode cursor advancement in `/compiler/sec4-cli/src/main.rs` inside `run_lasm_cluster_accept_loop(...)`.
- Cursor-next selection now does the following when relay pool is degraded (`relay_all_senders_live == false`):
  - keeps cursor pinned to the same shard for `relay_live_sender_count == 1`,
  - keeps existing dual-live alternation behavior for `relay_live_sender_count == 2`,
  - for `relay_live_sender_count > 2`, skips dead wrapped next-slot indices and advances directly to the next live shard when available.

## Why

In degraded multi-shard states, wrapped cursor advancement could step into known-dead relay slots, forcing avoidable disconnected primary sends and fallback work. That adds hot-path overhead under sustained load and increases fallback pressure.

## Result

- Degraded dispatch avoids dead-slot primary attempts in multi-live pools.
- Single-live degraded mode avoids redundant dead-slot cursor wrapping.
- Existing deterministic saturation/unavailable behavior is unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
