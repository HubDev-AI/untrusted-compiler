# 1095 M39 Slice: LASM DB Constraint Validation Error Classification

This slice extends LASM DB runtime error mapping so common constraint/type-shape failures are reported as deterministic validation errors.

## What changed

1. Extended `classify_lasm_db_runtime_error` validation detection:
   - sqlite:
     - `NOT NULL constraint failed`
     - `CHECK constraint failed`
   - postgres:
     - `violates not-null constraint`
     - `violates check constraint`
     - `invalid input syntax for`
2. Those failures now map to deterministic operation-specific validation envelopes:
   - `db.exec` -> `400 DB.EXEC_INVALID`
   - `db.execTx` -> `400 DB.EXEC_TX_INVALID`
   - `db.queryOne` -> `400 DB.QUERY_ONE_INVALID`
3. Added sqlite runtime regression:
   - internal exec with explicit `op = NULL` constraint violation
   - runtime now returns deterministic `HTTP 400` + `DB.EXEC_INVALID`
   - failed validation execution does not append runtime records

## Why

Before this change, many DB constraint/type failures surfaced as generic runtime failures.

Mapping these errors into deterministic validation envelopes gives clearer runtime diagnostics and more predictable client behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_maps_sqlite_not_null_exec_to_validation`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_maps_sqlite_duplicate_key_exec_to_conflict`
