# M39: LASM sqlite lock-retry runtime controls

## What changed

LASM sqlite runtime now supports configurable lock-contention retries for `database is locked` failures.

New env controls:

- `SEC4_RT_LASM_DB_SQLITE_LOCK_RETRY_MAX` (default `2`)
- `SEC4_RT_LASM_DB_SQLITE_LOCK_RETRY_DELAY_MS` (default `5`)

Behavior:

- on sqlite lock error, runtime performs bounded retries with optional delay,
- non-lock parameter errors still return immediately (no retry),
- `/db/records` telemetry now exposes effective sqlite retry settings:
  - `dbTimeoutsMs.sqliteLockRetryMax`
  - `dbTimeoutsMs.sqliteLockRetryDelayMs`.

## Why

Under concurrent load, sqlite lock errors can be transient. Bounded retry controls improve practical write-path resilience while preserving deterministic runtime behavior and operator observability.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
