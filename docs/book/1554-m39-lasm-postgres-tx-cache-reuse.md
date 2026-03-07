# M39: LASM Postgres Tx Client Cache Reuse

## What It Is

This slice moves LASM Postgres `db.execTx(...)` transaction handles onto the same cached client wrapper used by the thread-local Postgres path. Transactional routes no longer borrow raw `postgres::Client` instances that prepare every statement from scratch.

## Why It Exists

The first throughput fix removed the global LASM dynamic-state mutex from the hot Postgres `execTx` path. That fixed the collapse under load, but the canonical workbench route `POST /wb/tasks/with-comment` still paid repeated Postgres prepare costs because tx handles used raw clients without statement or placeholder caches.

That route is exactly the alpha benchmark path that proves real transactional Postgres behavior. Leaving it uncached would keep the main transactional proof-path artificially slow.

## How It Works Internally

Before this change:

- non-transaction Postgres work used `LasmPostgresThreadLocalClient`
- transaction work used raw `postgres::Client`
- pooled tx clients also stored raw clients
- every transactional prepared statement was prepared again on the hot path

After this change:

- `db_postgres_tx_clients` stores `LasmPostgresThreadLocalClient`
- the shared tx pool stores wrapped clients too
- detached tx clients keep placeholder and prepared-statement caches
- `run_lasm_postgres_exec_tx_on_client(...)` now uses cached placeholder counting and cached prepared statements
- stale prepared statements invalidate only the affected cached entry and retry once
- the workbench preflight harness no longer needs sec4-specific legacy payload allowances because the canonical LASM workbench endpoints now satisfy the shared `{ ok, status, traceId, timeMs, data }` contract directly

## Inputs, Outputs, and Constraints

Inputs:

- LASM Postgres runtime config
- transactional SQL template and params
- tx handle state from `db.tx(...)` / `db.execTx(...)`

Outputs:

- same HTTP and DB semantics as before
- lower warm-path cost for repeated transactional statements

Constraints:

- no language semantics changed
- transaction begin/commit/rollback behavior remains explicit and deterministic
- reconnect behavior is still conservative inside an active transaction

## Failure Modes and Diagnostics

Failure behavior is unchanged:

- invalid params still fail deterministically
- transaction begin/commit/rollback failures still surface as runtime DB errors
- stale prepared statement failures invalidate one cached entry and retry once
- reconnect-style recovery is still not attempted mid-transaction unless the caller explicitly recreates the transaction flow

## Example Usage

The canonical benchmark app route:

- `POST /wb/tasks/with-comment`

executes two `db.execTx(...)` calls on the same transaction source. With wrapped tx clients, the repeated task/comment inserts reuse the cached Postgres statement path instead of preparing from scratch on every hot request.

## Tradeoffs and Next Steps

Tradeoffs:

- tx clients now keep small per-client caches, so idle pooled tx clients hold slightly more memory
- the performance benefit matters only on repeated transactional query shapes, not one-off tx statements

Next steps:

- keep the scaling work focused on productizing the existing multi-instance runtime path
- keep benchmark-harness contract checks strict and backend-neutral now that the sec4 workbench path emits the canonical response shapes directly
