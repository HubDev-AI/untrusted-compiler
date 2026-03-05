# M39: LASM Cluster Relay Worker Snapshot Clone Elision

Date: 2026-02-22  
Milestone: M39 (proxy/runtime hot-path tuning)

## What Changed

- Added a relay-worker helper that loads worker-port snapshots from `ArcSwap` and only clones the `Arc<Vec<u16>>` when the snapshot pointer actually changes.
- Replaced hot-loop `load_full()` usage with pointer-checked `load()` + conditional clone in:
  - startup snapshot initialization
  - unhealthy-prune refresh branch
  - accept-batch selection refresh branch
- Preserved existing backend-selection semantics and snapshot-remap behavior.

## Why

The relay worker previously performed unconditional `ArcSwap::load_full()` clones in selection maintenance paths. Under sustained traffic this adds avoidable refcount churn in proxy hot loops even when worker topology is unchanged.

## Result

- Fewer unnecessary `Arc` clone operations in steady-state relay-worker loops.
- No behavior change in selection/fallback logic; only snapshot acquisition cost is reduced.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs`
- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
