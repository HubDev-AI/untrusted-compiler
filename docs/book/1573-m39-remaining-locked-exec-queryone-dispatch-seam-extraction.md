# M39: Remaining Locked Exec And QueryOne Dispatch Seam Extraction

## What it is

This slice moves the remaining sqlite/records-log `db.exec` and `db.queryOne` execution blocks out of top-level LASM runtime dispatch and into `lasm_db_client`.

Files:

- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- `compiler/sec4-cli/src/lasm_db_client/mod.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

## Why it exists

After `execTx` became fully helper-driven, the remaining dispatch-local DB ownership was concentrated in the non-Postgres `exec` and `queryOne` branches.

Those branches still owned:

- dynamic-state locking
- adapter execution
- record append/persist
- row-object return wiring

That was no longer a good split. Dispatch should resolve inputs and map HTTP responses. Adapter helpers should own the runtime lifecycle.

## What changed

Added:

- `run_lasm_non_postgres_exec_locked_operation(...)`
- `run_lasm_non_postgres_query_one_locked_operation(...)`

Those helpers now own the sqlite/records-log locked execution path.

`lasm_db_runtime_dispatch.rs` now just:

- resolves db/template/params inputs
- calls the helper
- maps deterministic success/error envelopes

## Cleanup in the same slice

Removed:

- `records::*` public re-export from `lasm_db_client/mod.rs`

Reason:

- after the move, record helpers were internal to `operations.rs`
- keeping the re-export made the module surface look wider than it really was and produced a warning

## Validation

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

Both passed.

## Tradeoffs

Pros:

- smaller top-level dispatch
- more consistent helper ownership across adapters
- easier next-step extraction on tx-control helpers

Cost:

- more orchestration now depends on helper APIs staying disciplined

That trade is correct, because DB adapter/runtime behavior belongs in `lasm_db_client`, not in the top-level HTTP dispatch file.

## Next

Continue the same pattern on the remaining tx-control seams:

- tx-handle allocation
- tx-handle resolution/cleanup ownership
- adjacent helper/package boundaries around DB runtime control flow
