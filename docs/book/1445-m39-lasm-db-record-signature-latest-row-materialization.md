# M39: LASM DB Signature Latest-Row Materialization for Records-Log QueryOne

Date: 2026-02-22  
Milestone: M39 (records-log fallback lookup/materialization)

## What Changed

- Extended LASM dynamic DB state with `db_latest_record_by_signature`:
  - key: `db + template + params`,
  - value: latest retained matching DB record.
- Kept latest-record map in sync for all records-log history transitions:
  - bootstrap from retained history,
  - append updates latest entry,
  - overflow eviction refreshes/removes entries only for affected signatures.
- Updated records-log `db.queryOne` fallback materialization:
  - fallback still creates a deterministic `queryOne` record in history,
  - returned `rowObject` now reflects the latest matching retained source record metadata.

## Why

Before this slice, records-log fallback match was O(1), but the returned row metadata mirrored the synthetic `queryOne` record instead of the latest matching retained DB record. That weakened realism for fallback row semantics.

## Result

- Records-log fallback keeps O(1) signature lookup behavior.
- Returned `rowObject` now carries latest retained matched record metadata.
- Overflow handling remains incremental and deterministic.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 append_db_record_enforces_capacity_by_dropping_oldest`
- `cargo test -p sec4 append_db_record_keeps_signature_when_duplicate_survives_overflow`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
