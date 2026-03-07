# sec4-lasm-workbench app

Canonical LASM implementation of the workbench task/comment app on the `sec4`
LASM runtime. Benchmark runners consume this service, but the route surface and
smoke contract are the primary operator-facing entrypoint.

## Routes

1. `GET /health`
2. `POST /wb/setup`
3. `POST /wb/tasks`
4. `POST /wb/tasks/with-comment`
5. `POST /wb/tasks/with-comment-tx`
6. `POST /wb/tasks/:id/comments`
7. `GET /wb/tasks/:id`
8. `GET /wb/tasks`
9. `GET /wb/records`

## Run

Postgres-backed, using the repo-local runtime env alias:

```bash
SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE=infra/local-postgres/.runtime.env \
cargo run -q -p sec4 -- run \
  --path benchmark-suite/services/sec4-lasm-workbench \
  --backend lasm \
  --db-adapter postgres \
  --port 18088
```

This canonical app is Postgres-only for operator/tutorial/benchmark flows. Do not use sqlite fallback here.

## Smoke tests

```bash
benchmark-suite/services/sec4-lasm-workbench/smoke.sh
```

Public canonical JSON flow:

```bash
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

## Public route shape

`POST /wb/tasks`

```json
{
  "title": "Task title",
  "description": "Task description",
  "status": "open",
  "priority": 3,
  "labels": ["api", "urgent"]
}
```

`POST /wb/tasks/with-comment`

```json
{
  "task": {
    "title": "Task title",
    "description": "Task description",
    "status": "in_progress",
    "priority": 4
  },
  "comment": {
    "body": "first note"
  }
}
```

`POST /wb/tasks/with-comment-tx`

- Same public JSON contract as `POST /wb/tasks/with-comment`
- Uses an explicit multi-step `db.execTx(...)` chain internally instead of the one-statement path
- This is the canonical service-level proof that repeated tx-handle reuse works in the real LASM path

`POST /wb/tasks/:id/comments`

```json
{
  "body": "follow-up"
}
```

`GET /wb/tasks/:id`

- No internal `row_schema` query flag required.

`GET /wb/tasks`

- Public query filters:
  - `status`
  - `priorityMin`
  - `priorityMax`
  - `label`
  - `limit`
  - `offset`
- Invalid public inputs now fail before DB execution with deterministic `400` workbench envelopes:
  - invalid `title` / `status` / `priority` / `labels`
  - invalid nested `task` / `comment` payloads
  - invalid list query filters (`status`, `priorityMin`, `priorityMax`, `limit`, `offset`)

## Notes

1. Mutating routes require `Authorization: Bearer token123`; the smoke helper keeps that deterministic default and only changes it if `BENCH_WORKBENCH_AUTH_TOKEN` is set explicitly.
2. `smoke.sh` and `smoke-public.sh` require Postgres. They prefer an already-configured DSN source (`BENCH_WORKBENCH_PG_DSN`, `SEC4_DB_ALPHA_DB_POSTGRES_DSN`, `SEC4_RT_LASM_DB_POSTGRES_DSN`, the DSN-file aliases, or the runtime-env-file aliases). If no explicit source is set, they try repo-local `infra/local-postgres/.runtime.env` and `.env`, and fail deterministically when local Postgres is not reachable.
3. `smoke-public.sh` exercises the public JSON contract: labels on create, one-statement transactional create-with-comment, explicit multi-step `execTx` create-with-comment, comment create, filtered list, deterministic validation failures on invalid create/list inputs, and deterministic `TASK.NOT_FOUND` on comment writes against a missing task id.
4. DB parameter payloads still accept the alpha query-string JSON contract (`params`, `task_params`, `comment_params`) for cross-backend parity, but the LASM service now also synthesizes those internals from the public JSON/query shape for operator-facing use.
5. `GET /wb/tasks` returns a real ordered list window from the active DB adapter through a single aggregated `db.queryOne` contract row, so the public JSON envelope exposes `items`, `count`, `limit`, and `offset` without backend-specific route branching.
