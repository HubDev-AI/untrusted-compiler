# 129 M7 Slice: DB Sink Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for DB sink invocation shapes.

## What it is

Added semantic argument-shape checks for DB sink calls:
- `db.exec`:
  - allowed forms: `(capability, query)` or `(ctx, capability, query)`.
- `db.execTx`:
  - allowed forms: `(tx, query)` or `(ctx, tx, query)`.
- `db.queryOne`:
  - allowed forms: `(capability, query, rowSchema)` or `(ctx, capability, query, rowSchema)`.

Malformed calls emit `E4001` tagged with `security` + `sink`.

## Why it exists

DB calls previously tolerated missing query arguments in several fixtures. That weakens sink intent and makes SQL safety hardening harder to stage cleanly.

This slice enforces explicit query-bearing call shapes without yet requiring query type `SqlQuery`.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_db_query_call_shapes(...)`,
2. recognized DB sink families via canonical call-name helpers,
3. validated per-sink allowed arities (compact and context-first forms),
4. emitted a consistent `E4001` shape diagnostic with fix-oriented notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures for missing-query DB calls
  - diagnostic-tag coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
  - DB/fs/net CLI integration test in `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of DB sink calls that omit required query/schema arguments,
  - updated semantic fixtures to valid query-bearing forms.
- Constraint:
  - this slice is call-shape hardening only; strict typed `SqlQuery` sink enforcement is staged separately.

## Failure modes and diagnostics

Examples:
- `db.exec(cap)` -> `E4001` (`db sink call has invalid argument shape`).
- `db.queryOne(netCapOnly)` -> shape error (and capability mismatch when applicable).

## Example usage

```ut
fn persist(ctx: Ctx, db: DbCap, tx: TxCap, q: SqlQuery) effects { db.write, db.read, db.tx } -> Int {
  db.exec(ctx, db, q);
  db.execTx(tx, q);
  db.queryOne(db, q, "RowSchema");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: malformed legacy fixtures now produce earlier shape diagnostics (sometimes alongside capability diagnostics).
- Next:
  - enforce typed-query-only DB sinks (`SqlQuery`) as part of M8 typed-sink constraints,
  - align SQL sink quick-fix suggestions with canonical call shapes.
