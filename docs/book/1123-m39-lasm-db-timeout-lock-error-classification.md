# 1123 M39 Slice: LASM DB Timeout/Lock Error Classification

This slice improves LASM DB runtime error envelopes for timeout and lock-contention failures.

## What changed

1. Extended DB runtime error classification in `compiler/sec4-cli/src/lasm_db_runtime_common.rs`:
   - Postgres statement timeout (`canceling statement due to statement timeout`) now maps to timeout-class deterministic envelopes.
   - Postgres lock timeout and sqlite lock contention (`database is locked`) now map to conflict-class deterministic envelopes.
2. Added deterministic timeout/lock codes by operation:
   - timeout: `DB.EXEC_TIMEOUT`, `DB.EXEC_TX_TIMEOUT`, `DB.QUERY_ONE_TIMEOUT`
   - lock timeout: `DB.EXEC_LOCK_TIMEOUT`, `DB.EXEC_TX_LOCK_TIMEOUT`, `DB.QUERY_ONE_LOCK_TIMEOUT`
3. Added focused unit coverage for classifier behavior:
   - statement-timeout mapping path
   - lock-timeout mapping path

## Why

Timeout and lock-contention failures are operationally different from generic runtime failures. Returning deterministic timeout/conflict envelopes gives operators and clients actionable behavior for retry/backoff handling.

## Validation

1. `cargo test -p sec4 classify_db_runtime_`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
