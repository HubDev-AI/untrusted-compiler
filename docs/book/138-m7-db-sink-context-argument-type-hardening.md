# 138 M7 Slice: DB Sink Context-Argument Type Hardening

This chapter documents a focused M7 follow-up that tightens context-first typing for DB sink intrinsics.

## What it is

Added semantic type enforcement for context-first DB sink calls:
- `db.exec(ctx, capability, query)` requires argument 1 to be `Ctx`.
- `db.execTx(ctx, tx, query)` requires argument 1 to be `Ctx`.
- `db.queryOne(ctx, capability, query, rowSchema)` requires argument 1 to be `Ctx`.

Violations now emit `E4001` with `security` + `sink` tags.

## Why it exists

DB sink call-shape hardening already constrained arity, but context-first forms still accepted non-context placeholders in slot 1. This update aligns DB sink context semantics with explicit typed trust-boundary rules.

## How it works internally

In `enforce_db_query_call_shapes(...)`:
1. kept existing arity checks for sink contracts,
2. added context-first branches for 3/4-argument DB sink forms,
3. delegated to `enforce_db_sink_context_type(...)`,
4. validated `arg_types[0]` is `Ctx`,
5. emitted deterministic `E4001` diagnostics with found-type and canonical-usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_db_exec_context_argument_type.ai`
    - `invalid_db_exec_tx_context_argument_type.ai`
    - `invalid_db_query_one_context_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid context-first DB sink calls,
  - sink-tagged diagnostics consumable by editor/LSP tooling.
- Constraint:
  - this slice tightens semantic validation only; no runtime ABI changes.

## Failure modes and diagnostics

Examples:
- `db.exec(1, db, query)` ->
  - `E4001`: db sink context argument must be `Ctx`.
- `db.execTx(1, tx, query)` ->
  - `E4001`: db sink context argument must be `Ctx`.
- `db.queryOne(1, db, query, schema)` ->
  - `E4001`: db sink context argument must be `Ctx`.

## Example usage

```ailang
fn readUser(ctx: Ctx, db: DbCap, query: SqlQuery) effects { db.read } -> Int {
  db.queryOne(ctx, db, query, 1);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: context-first DB sink calls with placeholder slot-1 values now fail semantic checks.
- Next:
  - apply equivalent context-first type enforcement to net/fs/secret capability calls where slot-1 typing is still only arity-checked.
