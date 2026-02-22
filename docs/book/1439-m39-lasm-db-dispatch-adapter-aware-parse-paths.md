# M39: LASM DB Dispatch Adapter-Aware Parse Paths

Date: 2026-02-22  
Milestone: M39 (DB runtime dispatch overhead reduction)

## What Changed

- Added adapter-aware DB dispatch wiring:
  - LASM backend now resolves active DB adapter once from dynamic state at bootstrap,
  - runtime threads that adapter through connection processing and response materialization into DB dispatch.
- Updated DB dispatch entrypoint to receive active adapter explicitly.
- Made SQL parameter pre-parse adapter-specific in DB dispatch:
  - Postgres parser runs only on Postgres adapter paths,
  - SQLite parser runs only on SQLite adapter paths.
- Added debug assertions that lock-held state adapter matches bootstrap adapter for deterministic branch behavior.

## Why

DB dispatch previously parsed both Postgres and SQLite SQL params for every DB intrinsic operation, even though only one adapter is active for a given runtime process. That created avoidable per-request parse overhead.

## Result

- DB intrinsic runtime path now parses only relevant adapter params.
- Existing DB behavior and envelopes are preserved.
- Runtime call chain now has explicit adapter context for DB dispatch decisions.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
