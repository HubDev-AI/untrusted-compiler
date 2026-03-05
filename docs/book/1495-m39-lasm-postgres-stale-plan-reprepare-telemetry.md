# M39 Slice: LASM Postgres Stale-Plan Reprepare Telemetry

Date: 2026-02-26  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Added runtime counter `db_postgres_stale_plan_reprepare_total` in LASM dynamic DB state.
- Wired stale prepared-statement eviction helper to increment this counter whenever stale-plan recovery is triggered.
- Extended `/db/records` response telemetry:
  - `dbRetries.postgresStalePlanReprepareTotal`

## Why

- Stale-plan recovery is now automatic on Postgres schema drift, but operators also need deterministic visibility that recoveries are happening under live traffic.
- Exposing this counter in existing DB telemetry keeps observability on the same runtime inspection surface used for retries/cache tuning.

## Behavioral contract

- Counter starts at `0` on runtime bootstrap.
- Counter increments once per stale prepared-plan recovery attempt that evicts/reprepares cached statements.
- `DbListRecordsResponse` always includes the field under `dbRetries`, independent of active adapter.

## Validation

- `cargo test -p sec4 stale_prepare_error_detection_matches_runtime_sqlstate_marker`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used -- --exact`
