# node-workbench service

Node.js workbench backend with real Postgres-backed task/comment routes.

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
BENCH_WORKBENCH_PG_DSN='postgresql://bench:bench@127.0.0.1:5432/bench' PORT=18089 node benchmark-suite/services/node-workbench/server.mjs
```

## Smoke

```bash
benchmark-suite/services/node-workbench/smoke.sh
```

## Notes

1. Mutating routes require `Authorization: Bearer token123`.
2. This implementation uses `psql` CLI invocation for DB operations to keep the service dependency-light for benchmark scaffolding.
