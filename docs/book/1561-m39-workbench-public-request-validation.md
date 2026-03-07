# M39 Slice: Workbench Public Request Validation

## What It Is

The canonical LASM workbench app now validates its public JSON bodies and list-filter query parameters before any Postgres work starts.

## Why It Exists

The previous public workbench bridge translated friendly JSON into benchmark-era query-array params, but malformed public inputs could still fall through into DB-shaped failures or silent defaults. That made the canonical app less trustworthy as a real backend surface.

## How It Works Internally

`compiler/sec4-cli/src/main.rs` now runs a dedicated workbench request-validation hook ahead of DB-operation materialization:

1. Detect whether the route is using the public JSON contract or the old benchmark query contract.
2. For public JSON routes, require `Content-Type: application/json`, parse the body as an object, and reject malformed payloads immediately.
3. Validate route-specific fields for:
   - `POST /wb/tasks`
   - `POST /wb/tasks/with-comment`
   - `POST /wb/tasks/:id/comments`
   - `GET /wb/tasks`
4. Emit the canonical workbench error envelope directly instead of letting invalid values leak into Postgres execution.

The benchmark wire-format remains supported. The new validation is only for the operator-facing public contract.

## Inputs, Outputs, And Constraints

Validated public inputs:

1. Task create:
   - `title` must be a non-empty string
   - `description` must be a string when present
   - `status` must be `open|in_progress|done` when present
   - `priority` must be an integer `1..5` when present
   - `labels` must be an array of non-empty strings when present
2. Transactional task create:
   - `task` must be an object
   - `comment` must be an object
   - nested `task` fields follow the same task validation rules
   - nested `comment.body` must be a non-empty string
3. Comment create:
   - `body` must be a non-empty string
4. Task list:
   - `status` must be `open|in_progress|done` when present
   - `priorityMin`, `priorityMax`, `limit`, `offset` must be integers when present
   - `priorityMin <= priorityMax` when both are present

Error envelope:

```json
{
  "ok": false,
  "status": 400,
  "traceId": "rt-7",
  "timeMs": 1770000000000,
  "error": {
    "code": "VALIDATION.INVALID",
    "kind": "validation",
    "message": "priority must be an integer between 1 and 5"
  }
}
```

## Failure Modes And Diagnostics

1. Missing or wrong JSON content type:
   - `400`
   - `VALIDATION.INVALID`
   - `content-type must be application/json`
2. Malformed JSON:
   - `400`
   - `JSON.INVALID_SYNTAX`
   - `invalid JSON payload`
3. Body is not a JSON object:
   - `400`
   - `VALIDATION.INVALID`
   - `request body must be a JSON object`
4. Invalid task/comment/filter fields:
   - `400`
   - `VALIDATION.INVALID`
   - deterministic first-failure message

## Example Usage

Valid create:

```bash
curl -i \
  -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data '{"title":"Public Task","status":"open","priority":3,"labels":["api"]}' \
  http://127.0.0.1:8080/wb/tasks
```

Invalid create:

```bash
curl -i \
  -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data '{"title":"Bad","priority":9}' \
  http://127.0.0.1:8080/wb/tasks
```

Invalid list filter:

```bash
curl -i \
  'http://127.0.0.1:8080/wb/tasks?priorityMin=oops'
```

## Tradeoffs And Next Steps

Tradeoffs:

1. Validation currently lives in the LASM public bridge, not in the `.ut` workbench modules themselves.
2. The benchmark wire-format path is still present because the benchmark suite depends on it.

Next steps:

1. Tune the transactional `wb-tasks-with-comment` Postgres hot path on the same workload.
2. Rerun same-workload mode and cross-implementation comparisons after each hot-path change.
3. Keep using the canonical workbench app to surface remaining DB/runtime cleanup work before deeper scaling passes.
