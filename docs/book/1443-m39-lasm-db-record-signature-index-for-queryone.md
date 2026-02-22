# M39: LASM DB Record Signature Index for QueryOne

Date: 2026-02-22  
Milestone: M39 (records-log fallback lookup optimization)

## What Changed

- Added a DB record signature index to LASM dynamic state:
  - signature key: `db + template + params`,
  - maintained on bootstrap and incremental append.
- Updated append/overflow path behavior:
  - appends insert signature immediately,
  - overflow compaction rebuilds signature set from retained records.
- Updated records-log `db.queryOne` fallback:
  - replaced reverse history scan with direct signature-set membership check.

## Why

Records-log `db.queryOne` fallback previously scanned persisted DB record history under lock for every lookup. As history grows, this introduces avoidable O(n) lock-held work.

## Result

- Records-log query fallback matching is now O(1) membership check.
- Existing fallback match semantics are preserved.
- Compaction path keeps index consistent with retained history.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 append_db_record_enforces_capacity_by_dropping_oldest`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
