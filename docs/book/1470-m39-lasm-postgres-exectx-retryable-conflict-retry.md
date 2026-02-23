# M39: LASM Postgres `db.execTx` retryable conflict retry

## What changed

LASM Postgres transactional runtime path now retries `db.execTx` once when a transaction fails with retryable concurrency SQLSTATE:

- `40001` (serialization failure)
- `40P01` (deadlock detected)

Behavior details:

- retry happens on the same runtime client path as an immediate second attempt,
- existing reconnect-on-closed path remains unchanged,
- prepared/unprepared query flow remains unchanged.

Additionally, DB runtime classification now treats common Postgres serialization/deadlock messages as deterministic `409` conflict envelopes.

## Why

Under concurrent write pressure, one-shot transaction execution can fail on transient serialization/deadlock conflicts even when a retry would succeed immediately. This change improves practical runtime resilience for real DB workloads without changing language-level intrinsic semantics.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 retryable_tx_sqlstate_detection_matches_serialization_and_deadlock_codes`
- `cargo test -p sec4 --bin sec4 classify_db_runtime_postgres_retryable_conflict_as_conflict`
