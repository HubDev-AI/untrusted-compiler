# sec4-lasm service

Untrusted<T> benchmark service using the LASM backend at runtime.
It implements:
- `GET /ping`
- `GET /health`
- `POST /decode`
- `POST /users`
- `GET /users/:id`
- `GET /db/hot-write`
- `GET /db/hot-write-tx`
- `GET /db/hot-query-one?row_schema=<int64>`
- `GET /db/records`

## Run

```bash
cargo run -q -p sec4 -- run --path benchmark-suite/services/sec4-lasm --backend lasm --port 8080
```

## Smoke test

```bash
benchmark-suite/services/sec4-lasm/smoke.sh
```

## Notes

- This service exercises the same benchmark contract as `services/sec4` but executes through the LASM backend directly.
- Current LASM benchmark fixture materializes request-aware benchmark JSON contracts (`/decode`, `/users`, `/users/:id`) from LASM route schemas while preserving status-code behavior.
- It is intended for side-by-side C-runtime vs LASM benchmark orchestration.
- DB routes execute real DB intrinsics (`db.exec`, `db.execTx`, `db.queryOne`) and can be used with `--profile db-hot-write`, `--profile db-hot-write-tx`, or `--profile db-hot-query-one` in capacity probes.
- The `db-hot-query-one` profile auto-warms with `/db/hot-write` before readiness checks so queryOne resolves deterministically.
