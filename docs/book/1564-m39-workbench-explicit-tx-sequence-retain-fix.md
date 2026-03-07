# M39 Slice: Workbench Explicit Tx Sequence Retain Fix

## What It Is

The canonical LASM workbench now has a second real transactional create route:

- `POST /wb/tasks/with-comment-tx`

It uses the explicit multi-step transaction shape:

1. `let tx = db.tx(db)`
2. `db.execTx(tx, insert-task)`
3. `db.execTx(tx, insert-comment)`

This route exists to prove the real tx-sequence runtime path on the canonical app, without replacing the benchmark-default one-statement route.

## Root Cause

The failure was not in the planner for the valid `db.tx(db)` route shape.

The actual bug was in the indexed DB-operation sequence executor in:

- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

When a later `execTx` step reused an existing transaction handle by DB source, the sequence path:

1. replaced `txDb` with the resolved `tx` handle,
2. but forgot to mark that handle as retained for the rest of the sequence.

For adapters that finalize transactions when retain is missing, the first `execTx` committed/dropped the tx handle. The next `execTx` step then failed with:

- `VALIDATION.INVALID`
- `db.execTx transaction handle must come from db.tx`

## What Changed

Two things landed:

1. The workbench app got the explicit route:
   - `benchmark-suite/services/sec4-lasm-workbench/src/main.ut`
   - `benchmark-suite/services/sec4-lasm-workbench/src/workbench/tasks.ut`
2. The sequence runtime now retains reused tx handles by source:
   - `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

The runtime fix now does both when it resolves an existing tx handle from `sequence_tx_handles_by_source`:

1. writes the non-indexed `tx` header for the current step
2. writes `X-Sec4-Internal-Db-Tx-Sequence-Retain: 1`
3. records that tx handle in the active sequence handle set

That keeps the tx alive until sequence cleanup runs at the end.

## Validation

Validated with the real operator smoke, not only an internal unit case:

```bash
cargo build -p sec4
BENCH_SMOKE_PORT=18122 benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

Result:

- `public smoke /wb/tasks/with-comment-tx` now passes
- existing public workbench smoke remains green

## Why Keep Both Routes

They serve different purposes:

1. `POST /wb/tasks/with-comment`
   - benchmark-default hot path
   - one-statement SQL shape
2. `POST /wb/tasks/with-comment-tx`
   - explicit multi-step tx proof path
   - validates repeated `db.execTx(tx, ...)` reuse on the real service

That keeps the benchmark route stable while still proving the runtime sequence behavior the alpha needs.

## Next

1. rerun targeted LASM same-workload measurements after this runtime cleanup
2. continue closing remaining DB/runtime cleanup exposed by the canonical workbench app
3. only then spend time on deeper scaling/runtime tuning
