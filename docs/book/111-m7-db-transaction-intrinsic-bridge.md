# 111 M7 Slice: DB Transaction Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: adding transaction helper intrinsics for database capability workflows.

## DB Transaction Helpers

### What it is

Added intrinsic support for:
- `db.tx` / `db_tx`
- `db.execTx` / `db_exec_tx`

Lowered runtime ABI stubs:
- `sec4_rt_db_tx`
- `sec4_rt_db_exec_tx`

### Why it exists

The v0 stdlib surface includes a transaction capability path (`DbCap` -> `TxCap`) and transaction-scoped write calls. Without these bridges, transaction-shaped service code in spec-aligned APIs cannot compile through the C backend.

### How it works internally

- Added `db.tx` effect to semantic known effects.
- Added semantic intrinsic entries:
  - `db.tx` uses effect `db.tx`, requires `DbCap`, returns `TxCap`
  - `db.execTx` uses effect `db.write`, requires `TxCap`, returns `Unit`
- Extended SQL sink checks so `db.execTx` gets the same secret/untrusted sink-flow enforcement as other SQL calls.
- Added capability argument index handling for context-first and compact call forms.
- Added C rewrite mappings and runtime stubs.
- Extended security-map tagging and argument-role labeling:
  - `db.tx` -> `effect.db.tx`, `capability.db`
  - `db.execTx` -> `sink.sql.exec`, `effect.db.write`, `capability.tx`
- Extended SQL query extraction heuristics so transaction execution calls are included in SQL hygiene analysis.

### Inputs, outputs, and constraints

- Input: transaction helper calls in bootstrap code.
- Output: generated C calls to `sec4_rt_db_tx(...)` and `sec4_rt_db_exec_tx(...)`.
- Constraints:
  - runtime transaction behavior remains stubbed in M7.
  - this slice validates compile-path typing, effect checks, capability checks, and metadata tags.

### Failure modes and diagnostics

- Missing `db.tx` effect when calling `db.tx` emits effect diagnostics.
- Missing or wrong capability type emits capability diagnostics (`E2003`/`E2004` paths).
- Untrusted or secret values flowing into `db.execTx` query arguments emit SQL sink diagnostics.

### Example usage

```ut
fn txDemo(db: DbCap, tx: TxCap, query: SqlQuery) effects { db.tx, db.write } -> Int {
  db.tx(db);
  db.execTx(tx, query);
  0
}
```

### Tradeoffs and next steps

- This closes a major DB API-shape gap without adding runtime complexity yet.
- Next steps are behavior-level transaction semantics (begin/commit/rollback and deterministic error mapping) in runtime-focused milestones.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - runtime ABI assertions include `sec4_rt_db_tx` and `sec4_rt_db_exec_tx`
  - db/fs/net rewrite test now covers `db.tx` and `db.execTx`
- `compiler/sec4-cli/tests/json_output.rs`
  - db/fs/net `c-bin` integration fixture now uses `TxCap` and transaction helper calls
  - generated C assertions include transaction runtime symbols
