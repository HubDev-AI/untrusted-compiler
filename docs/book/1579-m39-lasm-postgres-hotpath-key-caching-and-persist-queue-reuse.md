# M39: LASM Postgres Hotpath Key Caching and Persist Queue Reuse

Date: 2026-03-10  
Milestone: M39 post-alpha runtime tuning  
Status: Implemented

## What changed

This slice reduces avoidable overhead in LASM + Postgres runtime paths without changing public behavior.

### 1) Thread-local Postgres config now carries precomputed keys

`LasmPostgresThreadLocalConfig` now stores:

- shared client-pool key
- schema ensure key

Runtime paths that repeatedly touched pool/schema state now reuse those keys directly.

Files:

- `compiler/sec4-cli/src/lasm_db_client/config.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`

### 2) Postgres params parsing now uses borrowed JSON values

Postgres param parsing was switched from value-clone-heavy paths to borrowed-value parsing:

- array/object/scalar param parse loops now consume `&serde_json::Value`
- DB prepare dispatch in `lasm_db_client` no longer clones `parsed_params` before adapter parse

Files:

- `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`
- `compiler/sec4-cli/src/lasm_db_client/operations.rs`

### 3) Postgres persist queue worker path now reuses buffers and prunes stale lock entries

Persist queue improvements:

- each queued task carries precomputed config key
- worker loop reuses one batch vector (`drain` processing), removing repeated batch allocations
- stale per-config lock entries are pruned on a bounded cadence when lock registry grows

Files:

- `compiler/sec4-cli/src/lasm_db_runtime_postgres_persist.rs`

## Why

The canonical DB-backed workbench workload spends significant time on repeated DB execution and persistence paths.  
These changes target CPU/allocation pressure in those paths while preserving deterministic runtime behavior and diagnostics.

## Validation

Executed after the slice:

```bash
cargo fmt --all
CARGO_INCREMENTAL=0 cargo check -p sec4 --quiet
```

Result: pass.

