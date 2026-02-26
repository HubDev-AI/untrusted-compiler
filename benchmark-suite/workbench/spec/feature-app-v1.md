# Feature App Contract v1 (Workbench)

## Summary

A task-board API with tasks, comments, labels, and one transactional creation flow.

## Runtime Contract

1. Bind address: `127.0.0.1:${PORT}`
2. JSON API only (`application/json` for request/response bodies where applicable)
3. Deterministic envelope:
   - success:
     - `{ "ok": true, "status": <int>, "traceId": "<string>", "timeMs": <int>, "data": <json> }`
   - error:
     - `{ "ok": false, "status": <int>, "traceId": "<string>", "timeMs": <int>, "error": { "code": "<string>", "kind": "<string>", "message": "<string>" } }`

## Auth Contract

1. Header `Authorization: Bearer token123` is accepted in benchmark mode.
2. Missing/invalid auth for protected routes returns:
   - HTTP `401`
   - `error.code = "AUTH.REQUIRED"`
   - `error.kind = "auth"`

## Data Model (Postgres)

Tables:

1. `wb_tasks`
   - `id` uuid primary key
   - `title` text not null
   - `description` text not null default ''
   - `status` text not null (`open|in_progress|done`)
   - `priority` int not null (`1..5`)
   - `created_at_ms` bigint not null
2. `wb_comments`
   - `id` uuid primary key
   - `task_id` uuid not null references `wb_tasks(id)`
   - `body` text not null
   - `created_at_ms` bigint not null
3. `wb_labels`
   - `task_id` uuid not null references `wb_tasks(id)`
   - `name` text not null
   - primary key (`task_id`, `name`)

## Endpoints

### 1) `POST /wb/tasks`

Auth required: yes  
Body:

```json
{
  "title": "string",
  "description": "string",
  "status": "open",
  "priority": 3,
  "labels": ["api", "urgent"]
}
```

Behavior:

1. Validate required `title` non-empty, status in enum, priority `1..5`.
2. Insert task row + labels rows.
3. Return `201` with `{ id }`.

### 2) `POST /wb/tasks/with-comment`

Auth required: yes  
Body:

```json
{
  "task": {
    "title": "string",
    "description": "string",
    "status": "open",
    "priority": 3
  },
  "comment": {
    "body": "string"
  }
}
```

Behavior:

1. Atomic DB transaction:
   - insert task
   - insert initial comment bound to created task
2. On any failure, rollback both inserts.
3. Return `201` with `{ taskId, commentId }`.

### 3) `POST /wb/tasks/:id/comments`

Auth required: yes  
Body:

```json
{
  "body": "string"
}
```

Behavior:

1. Validate task exists.
2. Insert comment row.
3. Return `201` with `{ id }`.

### 4) `GET /wb/tasks/:id`

Auth required: no  
Behavior:

1. Return task + labels + comments count.
2. If not found:
   - HTTP `404`
   - `error.code = "TASK.NOT_FOUND"`
   - `error.kind = "missing_dependency"`

### 5) `GET /wb/tasks`

Auth required: no  
Query params:

1. `status` optional (`open|in_progress|done`)
2. `priorityMin` optional int
3. `priorityMax` optional int
4. `label` optional string
5. `limit` optional int default 20 max 100
6. `offset` optional int default 0

Behavior:

1. Deterministic ordering by `created_at_ms DESC`, then `id DESC`.
2. Return `{ items, count, limit, offset }`.

## Alpha Wire-Format Profile (current benchmark lanes)

For deterministic cross-backend parity in this alpha slice, implementations must accept query-string JSON-array payloads in addition to backend-native field forms.

1. `POST /wb/tasks`:
   - `params=[id,title,description,status,priority,created_at_ms]`
2. `POST /wb/tasks/with-comment`:
   - `task_params=[id,title,description,status,priority,created_at_ms]`
   - `comment_params=[id,task_id,body,created_at_ms]`
3. `POST /wb/tasks/:id/comments`:
   - `params=[id,task_id,body,created_at_ms]`
   - `task_id` from params must match route task id.
4. `GET /wb/tasks`:
   - `params=[status,limit,offset]`

These alpha wire-format keys are benchmark harness contracts, not language semantics. They can evolve once a shared benchmark request-generator profile is introduced.

## Validation Error Contract

Validation failures return:

1. HTTP `400`
2. `error.code = "VALIDATION.INVALID"`
3. `error.kind = "validation"`
4. Deterministic message indicating first failing field.

## Benchmark Mix (initial)

1. `POST /wb/tasks`: 25%
2. `POST /wb/tasks/:id/comments`: 20%
3. `POST /wb/tasks/with-comment`: 15%
4. `GET /wb/tasks/:id`: 20%
5. `GET /wb/tasks`: 20%

## Non-Goals

1. Full-text search engine integration.
2. Multi-tenant authorization model.
3. Background jobs.
