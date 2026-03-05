# sec4 service

Untrusted<T> benchmark service implementing:
- `GET /ping`
- `GET /health`
- `POST /decode`
- `POST /users`
- `GET /users/:id`
- `GET /db/hot-write`
- `GET /db/hot-write-tx`
- `GET /db/hot-query-one`
- `GET /db/records`

## Run

```bash
benchmark-suite/services/sec4/build.sh
PORT=8080 benchmark-suite/services/sec4/sec4-bench-server
```

## Smoke test

```bash
benchmark-suite/services/sec4/smoke.sh
```

## Notes

- This service compiles Untrusted<T> source to generated C and links a benchmark-specific runtime adapter.
- The adapter provides contract-oriented HTTP/JSON behavior suitable for benchmark harness comparability.
- The runtime adapter now supports benchmark DB intrinsics (`db.exec`, `db.execTx`, `db.queryOne`) and records endpoints for side-by-side C vs LASM probe runs.
