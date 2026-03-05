# M39 Slice: LASM Postgres Stale Plan Reprepare Recovery

Date: 2026-02-26  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Extended stale prepared-statement detection to include Postgres schema-drift execution error text:
  - `cached plan must not change result type`
- Wired this detection through the shared stale-prepare path used by Postgres prepared execution/retry flow.
- Existing stale-plan handling behavior (statement-cache eviction + reprepare retry) now also applies to this schema-drift class for:
  - `db.exec`
  - `db.execTx`
  - `db.queryOne`

## Why

- Postgres can invalidate cached prepared plans when row/result shape changes after DDL.
- The runtime already recovered from missing prepared statements (`sqlstate=26000`) but did not recover from stale-plan result-shape errors, causing avoidable deterministic failures on otherwise valid queries.

## Behavioral contract

- When prepared execution hits a stale-plan error (`cached plan must not change result type`), LASM runtime treats it as recoverable stale statement state.
- Runtime evicts the cached statement and retries with a freshly prepared statement.
- Non-stale SQL/runtime failures keep existing deterministic error classification behavior.

## Validation

- `cargo test -p sec4 stale_prepare_error_detection_matches_runtime_sqlstate_marker`
