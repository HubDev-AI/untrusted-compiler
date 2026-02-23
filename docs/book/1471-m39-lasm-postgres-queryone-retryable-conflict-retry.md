# M39: LASM Postgres `db.queryOne` retryable conflict retry

## What changed

LASM Postgres query-one runtime path now retries once when initial execution fails with retryable concurrency SQLSTATE:

- `40001` (serialization failure)
- `40P01` (deadlock detected)

Behavior details:

- retry uses the existing prepared query path,
- reconnect-on-closed behavior remains unchanged,
- single-statement and row-returning query guards remain unchanged.

## Why

Even read/materialization query paths can hit retryable concurrency failures on some transactional workloads. A one-shot retry improves runtime resilience without changing intrinsic behavior surface.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 retryable_tx_sqlstate_detection_matches_serialization_and_deadlock_codes`
