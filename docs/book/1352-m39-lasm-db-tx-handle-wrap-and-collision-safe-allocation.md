# M39: LASM DB Tx-Handle Wrap and Collision-Safe Allocation

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime + DB hardening)

## What Changed

- Updated `$REPO_ROOT/compiler/sec4-cli/src/lasm_db_runtime_common.rs`:
  - `allocate_lasm_db_tx_handle` now allocates by probing for a vacant positive handle slot,
  - allocator wraps from `i64::MAX` back to `1` and advances `next_db_tx_handle` accordingly,
  - allocator no longer relies on monotonic saturating increment that can eventually pin on the same terminal handle value.

## Why

Long-lived runtimes and high tx-handle churn need deterministic non-overwriting allocation behavior. A wrap/collision-safe allocator preserves uniqueness and capacity semantics even near numeric bounds or sparse-handle reuse scenarios.

## Result

- Deterministic tx-handle uniqueness under wrap-around conditions.
- No silent overwrite risk when handle counter nears `i64::MAX`.
- Existing tx-handle capacity guards remain unchanged.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_with_cli_flag_when_sqlite_adapter`
