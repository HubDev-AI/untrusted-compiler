# M39: LASM Status Writer Snapshot Clone Elision

Date: 2026-02-22  
Milestone: M39 (scaling/runtime hot-path tuning)

## What Changed

- Updated LASM cluster status writer to keep a selected worker-port snapshot across loop ticks.
- Replaced unconditional snapshot cloning (`load_full`) with pointer-checked `ArcSwap::load()` and clone-on-change behavior.
- Worker-count derivation now reuses cached selected snapshot length on unchanged topology.

## Why

Status writer loop previously cloned worker-port snapshots every tick, even when topology was unchanged. This added avoidable refcount/clone churn on a periodic hot path.

## Result

- No snapshot clone churn on unchanged topology ticks.
- Snapshot clone occurs only when worker-port topology actually changes.
- Existing status JSON semantics and unchanged-snapshot write elision behavior remain intact.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
