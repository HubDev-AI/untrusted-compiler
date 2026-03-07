# M39: Remaining ExecTx Locked Dispatch Seam Extraction

## What it is

This slice finishes the `db.execTx` dispatch reduction by moving the remaining sqlite/records-log lifecycle out of top-level LASM runtime dispatch and into `lasm_db_client`.

Files:

- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

## Why it exists

After the earlier Postgres extraction, `db.execTx` was still structurally split:

- Postgres path went through a DB-client helper
- sqlite/records-log path still kept a large stateful block in dispatch

That remaining block still owned:

- adapter execution
- commit/rollback decisions
- tx-handle cleanup
- record append/persist wiring

The correct end state is one consistent rule across adapters: dispatch resolves request inputs and HTTP envelopes, helper modules own adapter execution and stateful record lifecycle.

## How it works internally

Added:

- `run_lasm_non_postgres_exec_tx_locked_operation(...)`

That helper now owns the locked sqlite/records-log `execTx` flow:

1. lock dynamic state
2. verify adapter state
3. run adapter execution
4. handle rollback on failure
5. handle commit/tx-handle cleanup on success
6. append/persist runtime record

`lasm_db_runtime_dispatch.rs` now keeps:

- tx-source resolution
- error-envelope mapping
- success response/header emission

## Important cleanup in the same slice

`LasmDbExecTxOperationResult::Postgres` was removed.

Why:

- `run_lasm_db_exec_tx_operation(...)` is no longer the place where Postgres `execTx` is supposed to run.
- Postgres now has its own dedicated unlocked helper path.

Leaving that dead variant in place would keep misleading ownership in the API.

## Validation

Used for this slice:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

Both passed.

## Tradeoffs

Pros:

- smaller top-level dispatch
- clearer adapter ownership
- `execTx` now follows one structural pattern across adapters

Cost:

- more logic now lives in helper modules, so mistakes in helper boundaries can break multiple adapter paths at once

That trade is still correct, because the DB-client package is exactly where adapter-specific runtime ownership should live.

## Next

Continue the same pattern on the smaller remaining locked DB dispatch ownership, especially the non-Postgres `exec` / `queryOne` branches and adjacent tx-control helpers.
