# M39: Postgres Unlocked Dispatch Seam Extraction

## What it is

This slice moves the remaining unlocked Postgres non-transactional execution seam out of top-level runtime dispatch and behind the DB-client module boundary.

Files:

- `compiler/sec4-cli/src/lasm_db_client/records.rs`
- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

## Why it exists

After the canonical LASM/Postgres workbench app was stabilized and tuned, the main remaining structural smell was in `lasm_db_runtime_dispatch.rs`:

- `db.exec` and `db.queryOne` on Postgres still mixed
  - request/param resolution,
  - adapter-state checking,
  - config building,
  - unlocked Postgres execution,
  - record append/persist,
  - response shaping

That was too much adapter-specific runtime logic in the top-level dispatch layer.

## How it works internally

### 1. Generic DB record helpers moved to DB-client module

Added:

- `lasm_db_client/records.rs`

This module now owns:

- `persist_lasm_db_record_with_capacity_guard(...)`
- `allocate_lasm_db_runtime_record(...)`
- `append_lasm_db_record_in_memory_with_compaction_snapshot(...)`

Those helpers were previously local to `lasm_db_runtime_dispatch.rs`.

### 2. Unlocked Postgres helpers now live in DB-client operations

Added focused helpers in:

- `lasm_db_client/operations.rs`

New helpers:

- `run_lasm_postgres_exec_unlocked_operation(...)`
- `run_lasm_postgres_query_one_unlocked_operation(...)`

Those helpers now own the adapter-specific path:

1. lock dynamic state
2. verify Postgres adapter state
3. build thread-local config
4. unlock and execute thread-local Postgres work
5. re-lock and append runtime record
6. persist Postgres record after unlock

### 3. Dispatch now keeps orchestration only

`lasm_db_runtime_dispatch.rs` now:

- resolves request headers/template/params
- maps helper results to deterministic HTTP envelopes
- preserves row-size and row-column enforcement for `queryOne`

That keeps dispatch on orchestration + response shaping instead of direct adapter execution ownership.

## Inputs, outputs, and constraints

Inputs:

- canonical LASM workbench Postgres requests
- internal DB operation markers for `exec` and `queryOne`

Outputs:

- same deterministic HTTP responses
- same runtime record append/persist behavior
- smaller top-level dispatch surface

Constraints:

- no public contract change
- no benchmark workload change
- no change to error codes/status mappings

## Failure modes and diagnostics

The extraction preserves the existing categories:

- dynamic-state unavailable -> same internal error envelope
- adapter mismatch -> `DB.ADAPTER_MISMATCH`
- preparation mismatch -> same internal preparse mismatch path
- query-one not found -> `404 DB.QUERY_ONE_NOT_FOUND`
- runtime execution failure -> same `classify_lasm_db_runtime_error(...)` path

## Example usage

The canonical public smoke still exercises the extracted paths through:

- `POST /wb/tasks`
- `POST /wb/tasks/with-comment`
- `POST /wb/tasks/:id/comments`
- `GET /wb/tasks/:id`
- `GET /wb/tasks`

Validation used for this slice:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

## Tradeoffs and next steps

This is a structural cleanup, not a new feature.

Tradeoffs:

- the DB-client layer now owns more runtime orchestration detail for Postgres
- dispatch becomes smaller and clearer, but result/error typing between layers is a bit more explicit

That is the right trade here because it keeps adapter-specific execution logic out of top-level dispatch.

Next step:

- continue extracting the remaining LASM DB adapter/package seams without changing language/runtime behavior
- keep the tuned canonical workbench app and benchmark scripts as the acceptance surface
