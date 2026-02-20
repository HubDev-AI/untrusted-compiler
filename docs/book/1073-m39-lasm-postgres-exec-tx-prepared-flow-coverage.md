# 1073 M39 Slice: LASM Postgres ExecTx Prepared Flow Coverage

This slice extends LASM Postgres integration coverage to lock the prepared `db.execTx` runtime path.

## What changed

1. Extended `run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available` to include a real `POST /db/exec-tx` request using placeholder params (`$1`, `[2]`).
2. Added deterministic response assertions for `execTx` metadata:
   - `recordId` sequencing
   - `op=execTx`
   - `tx` handle visibility
3. Updated record-list assertions to require both persisted operation kinds (`exec` and `execTx`) and count parity.

## Why

Prepared `db.execTx` is a core DB write path for LASM Postgres alpha behavior.

Locking it in the main Postgres command integration flow reduces regression risk while keeping validation scope focused.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
