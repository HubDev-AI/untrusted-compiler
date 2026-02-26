# go-workbench service

Go workbench backend with real Postgres-backed task/comment routes.

## Routes

1. `GET /health`
2. `POST /wb/setup`
3. `POST /wb/tasks`
4. `POST /wb/tasks/with-comment`
5. `POST /wb/tasks/:id/comments`
6. `GET /wb/tasks/:id`
7. `GET /wb/tasks`

## Run

```bash
BENCH_WORKBENCH_PG_DSN='postgresql://127.0.0.1:5432/postgres?sslmode=disable' PORT=18090 go run benchmark-suite/services/go-workbench
```

## Smoke

```bash
benchmark-suite/services/go-workbench/smoke.sh
```

## Notes

1. Mutating routes require `Authorization: Bearer token123`.
2. This bootstrap implementation uses `psql` CLI execution to keep setup minimal in the benchmark matrix lane.
