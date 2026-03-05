# sec4-lasm-workbench service

Feature-rich LASM benchmark workbench service with real DB intrinsic execution.

Implemented endpoints:

1. `GET /health`
2. `POST /wb/setup`
3. `POST /wb/tasks`
4. `POST /wb/tasks/with-comment`
5. `POST /wb/tasks/:id/comments`
6. `GET /wb/tasks/:id`
7. `GET /wb/tasks`
8. `GET /wb/records`

## Run

```bash
cargo run -q -p sec4 -- run --path benchmark-suite/services/sec4-lasm-workbench --backend lasm --db-adapter sqlite --db-base /tmp/sec4-wb-db --port 18088
```

## Smoke test

```bash
benchmark-suite/services/sec4-lasm-workbench/smoke.sh
```

## Notes

1. Mutating routes require `Authorization: Bearer token123` (auth helper enforcement).
2. DB parameter payloads are passed as query-string JSON text in this alpha slice (for deterministic runtime parsing across adapters).
3. `GET /wb/tasks` currently exposes a deterministic list-window probe via `db.queryOne` (single-row window sample), while full operation history is available via `GET /wb/records`.
