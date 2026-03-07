# M39 Slice: Workbench Response Id Fast Path And Cross-Runtime Rerun

## What It Is

The canonical LASM workbench now avoids a full JSON parse when success-envelope shaping only needs the first string id from a benchmark-era query array.

## Why It Exists

After fixing the request-side `task_params` + `comment_params` combine regression, the workbench path still deserialized query-array payloads again during response normalization just to recover task/comment ids for the canonical JSON envelope.

That was unnecessary work on every hot-path request.

## How It Works Internally

Two files changed:

1. `compiler/sec4-cli/src/lasm_request_template.rs`
   - exports a flat-array helper that reads one string element directly from a flat JSON array string
2. `compiler/sec4-cli/src/main.rs`
   - `lasm_workbench_task_id_from_query_array(...)` now uses that flat fast path first
   - falls back to the older `serde_json` parse only if the fast path cannot decode the value

This keeps the public/benchmark contracts unchanged while shaving response-shaping overhead from the `wb-tasks-with-comment` benchmark path.

## Inputs, Outputs, And Constraints

Optimized success-envelope extraction applies to flat array payloads such as:

- `params=[...]`
- `task_params=[...]`
- `comment_params=[...]`

Constraints:

1. The fast path is intentionally for flat arrays with scalar JSON entries.
2. String decoding still remains correct because the extracted quoted JSON string is decoded through `serde_json::from_str::<String>()`.
3. If the flat fast path fails, the old parse path still preserves behavior.

## Failure Modes And Diagnostics

1. Malformed flat-array input:
   - fast path returns `None`
   - code falls back to the previous parse behavior
2. Non-string first entry:
   - fast path returns `None`
   - fallback path decides whether the envelope can still extract a string id

No user-facing contract changed in this slice.

## Example Usage

The external contract is unchanged:

```bash
curl -i \
  -X POST \
  -H 'Authorization: Bearer token123' \
  "http://127.0.0.1:8080/wb/tasks/with-comment?task_params=%5B%22task-1%22%2C%22Task%22%2C%22desc%22%2C%22open%22%2C3%2C1700000000000%5D&comment_params=%5B%22comment-1%22%2C%22task-1%22%2C%22hello%22%2C1700000000001%5D"
```

The difference is internal: the success envelope now recovers `taskId` / `commentId` without full array deserialization on the fast path.

## Tradeoffs And Next Steps

Tradeoffs:

1. This is another specialized fast path aimed at the canonical benchmark contract.
2. It keeps a fallback parse path, so behavior stays conservative even if the fast scanner cannot decode a value.

Updated short benchmark result on the real Postgres-backed `wb-tasks-with-comment` workload:

- sec4-lasm: `256.64 req/s`, `p99 54.81ms`
- node: `10.58 req/s`, `p99 701.95ms`
- go: `59.03 req/s`, `p99 837.12ms`
- rust: `44.07 req/s`, `p99 947.71ms`

Next steps:

1. Continue DB/runtime cleanup exposed by the canonical app.
2. Reconcile the remaining gap between the plain matrix sec4-lasm run and the higher mode-compare single/fixed numbers.
3. Then keep pushing the scaling/runtime tuning path on the same workload.
