# 1094 M39 Slice: LASM DB Unique-Conflict Error Classification

This slice improves LASM DB runtime error contracts by classifying unique-constraint failures as deterministic conflict responses.

## What changed

1. Extended `classify_lasm_db_runtime_error`:
   - sqlite `UNIQUE constraint failed` and postgres `duplicate key value violates unique constraint` now map to conflict classification
   - status now returns `409`
   - operation-specific deterministic codes:
     - `DB.EXEC_CONFLICT`
     - `DB.EXEC_TX_CONFLICT`
     - `DB.QUERY_ONE_CONFLICT`
2. Added sqlite command integration regression:
   - first deterministic exec succeeds
   - duplicate-key exec on restart path returns `HTTP 409`
   - envelope now includes `DB.EXEC_CONFLICT`
   - failed conflict exec does not append extra runtime record entries

## Why

Previously, unique-key collisions surfaced as generic `500` DB failure envelopes.

Conflict-classification gives deterministic operator and client semantics for a common DB write failure mode.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_maps_sqlite_duplicate_key_exec_to_conflict`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
