# M39: LASM Postgres `db.exec` retryable conflict retry

## What changed

LASM Postgres execution path now retries `db.exec` once when initial execution fails with retryable concurrency SQLSTATE:

- `40001` (serialization failure)
- `40P01` (deadlock detected)

Behavior details:

- one immediate retry on the existing client path,
- keeps reconnect-on-closed behavior unchanged,
- keeps prepared/unprepared execution flow unchanged.

## Why

Retryable transaction conflicts can happen under concurrent write contention. This hardens `db.exec` behavior to align with existing conflict-retry posture for other DB runtime paths.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 retryable_tx_sqlstate_detection_matches_serialization_and_deadlock_codes`
