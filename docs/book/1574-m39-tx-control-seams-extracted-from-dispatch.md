# M39: Tx Control Seams Extracted From Dispatch

## What it is

This slice moves the remaining tx-control ownership out of top-level LASM DB dispatch and into `lasm_db_client`.

Files:

- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

## Why it exists

After `exec`, `queryOne`, and `execTx` became helper-driven across adapters, the remaining DB-runtime state still owned directly by dispatch was tx control:

- `db.tx` handle allocation
- `execTx` tx-handle binding resolution
- in-use conflict setup before helper execution

Those are runtime DB responsibilities, not HTTP dispatch responsibilities.

## What changed

Added helper-owned tx-control APIs:

- `run_lasm_db_tx_allocate_locked_operation(...)`
- `resolve_lasm_exec_tx_state_bindings_locked(...)`

Moved:

- `LasmExecTxSource`

from dispatch into the DB-client layer, so the tx-control contract now lives with the tx-control code.

## Resulting ownership split

`lasm_db_runtime_dispatch.rs` now:

- parses/validates request-facing tx markers
- maps deterministic HTTP envelopes

`lasm_db_client` now:

- allocates tx handles
- resolves tx source to live runtime state
- marks tx handles `in_use`
- returns structured control errors back to dispatch

## Validation

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

Both passed.

## Tradeoffs

Pros:

- dispatch is smaller and more regular
- tx-control logic now sits beside the rest of DB runtime ownership
- later tuning/debugging can focus on one DB-client package instead of bouncing between dispatch and helpers

Cost:

- the shared control error surface widened, so existing match sites had to be updated carefully

That trade is still correct. It localizes the hard stateful DB behavior into one package.

## Next

Use the cleaner helper-driven runtime surface for the next canonical same-workload benchmark rerun and scaling/runtime tuning pass.
