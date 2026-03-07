# M39: Postgres ExecTx Dispatch Seam Extraction

## What it is

This slice moves the remaining Postgres `db.execTx` execution seam out of top-level runtime dispatch and behind the DB-client boundary.

Files:

- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

## Why it exists

After moving unlocked Postgres `db.exec` and `db.queryOne` behind DB-client helpers, the last large adapter-specific block in `lasm_db_runtime_dispatch.rs` was `db.execTx`.

That branch still mixed:

- tx-source resolution
- Postgres config build
- tx-client take/return/discard logic
- transactional execution
- record append/persist
- HTTP response mapping

The right split is the same one already proven on the other Postgres operations.

## How it works internally

Added a new DB-client helper:

- `run_lasm_postgres_exec_tx_unlocked_operation(...)`

That helper now owns the Postgres-specific runtime lifecycle:

1. lock dynamic state
2. verify Postgres adapter state
3. build Postgres thread-local config
4. take any retained tx client
5. execute the transactional Postgres operation
6. update tx-handle state / return-or-discard client
7. append runtime record
8. persist Postgres record after unlock

`lasm_db_runtime_dispatch.rs` now keeps only:

- request/template/tx-source resolution
- deterministic HTTP error mapping
- success envelope/header emission

## Inputs, outputs, and constraints

Inputs:

- canonical workbench `db.execTx` runtime path
- resolved tx source (`db.tx(...)` or explicit tx handle)
- prepared Postgres params

Outputs:

- same `db.execTx` response envelope
- same tx result header behavior
- same deterministic tx-handle semantics

Constraints:

- no change to public route contract
- no change to transactional semantics
- no benchmark workload change

## Failure modes and diagnostics

Behavior stays the same for:

- state unavailable
- adapter mismatch
- preparation mismatch
- runtime tx failure

Dispatch still maps those failures to the same deterministic HTTP envelopes as before.

## Example usage

The canonical workbench public smoke still exercises the tx path through:

- `POST /wb/tasks/with-comment-tx`

Validation used for this slice:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

## Tradeoffs and next steps

This is a structural refactor, not a feature.

Tradeoffs:

- DB-client operations now own more of the tx lifecycle
- dispatch becomes smaller and more regular

That is the correct trade because it keeps adapter-specific execution out of the top-level runtime file.

Next:

- continue the remaining DB adapter/package extraction seams with the same pattern
- keep the current tuned workbench app as the acceptance surface
