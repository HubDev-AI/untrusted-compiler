# sec4-workbench service

Feature-app benchmark workbench lane using `sec4` C backend runtime execution.

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
SEC4_RT_DB_BASE=/tmp/sec4-workbench-db cargo run -q -p sec4 -- run --path benchmark-suite/services/sec4-workbench --backend c --port 18092
```

## Smoke

```bash
benchmark-suite/services/sec4-workbench/smoke.sh
```

## Notes

1. Mutating routes require `Authorization: Bearer token123`.
2. This lane uses C backend runtime envelopes as currently emitted (`ok/status/traceId/timeMs/data`).
3. DB route contract is shared with `sec4-lasm-workbench` route shapes for matrix parity.
