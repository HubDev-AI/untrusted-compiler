# M39 Slice: Workbench With-Comment Flat Query Combine Hot Path

## What It Is

The canonical LASM workbench `POST /wb/tasks/with-comment` hot path now keeps the one-statement SQL route, but replaces the expensive query-array combine step with a cheap flat-array combiner.

## Why It Exists

The first optimization attempt moved the route from two `db.execTx(...)` calls to one `db.exec(...)` statement, but the real regression was not the SQL itself. The regression came from synthesizing a combined `params` array on every benchmark request by fully parsing `task_params` and `comment_params` JSON arrays and then serializing them again.

That per-request JSON work cut single-instance throughput roughly in half on the real Postgres workload.

## How It Works Internally

`compiler/sec4-cli/src/lasm_request_template.rs` now does two different things for `/wb/tasks/with-comment`:

1. Public JSON requests still build `params`, `task_params`, and `comment_params` from the public payload object.
2. Benchmark/query-contract requests now build `params` from existing `task_params` and `comment_params` strings using a lightweight flat-array scanner.

The flat-array scanner:

1. Accepts only a flat JSON array string.
2. Scans elements without deserializing to `serde_json::Value`.
3. Preserves raw scalar JSON fragments.
4. Reassembles the combined array needed by the one-statement SQL route.

The route itself remains in:

`benchmark-suite/services/sec4-lasm-workbench/src/workbench/tasks.ut`

and still executes one atomic SQL statement:

- insert task
- insert initial comment

inside one `db.exec(...)` call.

## Inputs, Outputs, And Constraints

Accepted internal benchmark inputs remain unchanged:

- `task_params=[id,title,description,status,priority,created_at_ms]`
- `comment_params=[id,task_id,body,created_at_ms]`

Synthesized internal value:

- `params=[task_id,title,description,status,priority,task_created_at_ms,comment_id,comment_body,comment_created_at_ms]`

Constraints:

1. The fast combiner is intentionally for flat scalar arrays only.
2. If the fast path cannot parse the arrays, synthesis fails cleanly instead of inventing malformed DB params.
3. Public JSON contract behavior is unchanged.

## Failure Modes And Diagnostics

1. Invalid flat-array query payloads:
   - combined `params` synthesis fails
   - route falls back to normal validation/runtime failure behavior
2. Public JSON path:
   - still uses the existing deterministic public validation and payload-building path
3. The attempted revert back to multi-`execTx` on this route exposed another runtime issue:
   - repeated `db.execTx(tx, ...)` on the same workbench route currently does not preserve the transaction handle the way this route needs
   - that is recorded as follow-up DB/runtime cleanup, not left hidden

## Example Usage

Benchmark-compatible request contract stays the same:

```bash
curl -i \
  -X POST \
  -H 'Authorization: Bearer token123' \
  "http://127.0.0.1:8080/wb/tasks/with-comment?task_params=%5B%22task-1%22%2C%22Task%22%2C%22desc%22%2C%22open%22%2C3%2C1700000000000%5D&comment_params=%5B%22comment-1%22%2C%22task-1%22%2C%22hello%22%2C1700000000001%5D"
```

Public JSON contract also stays the same:

```bash
curl -i \
  -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data '{"task":{"title":"Task","status":"open","priority":3},"comment":{"body":"hello"}}' \
  http://127.0.0.1:8080/wb/tasks/with-comment
```

## Tradeoffs And Next Steps

Tradeoffs:

1. The route stays on the one-statement SQL shape because it is the reliable path today.
2. The fast array combiner is intentionally specialized to the benchmark contract instead of being a general JSON engine.

Next steps:

1. Inspect and fix the LASM multi-`db.execTx` sequence path exposed by this route experiment.
2. Keep the canonical benchmark workload on the one-statement route until the tx-handle reuse path is proven correct.
3. Continue DB/runtime cleanup exposed by the canonical app before deeper scaling work.
